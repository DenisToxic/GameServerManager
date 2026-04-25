#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use http::StatusCode;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::Config,
    docker::DockerClient,
    error::AppError,
    models::{
        BackendCreateContainerRequest, ContainerActionRequest, ContainerLogsResponse,
        ContainerStatusResponse, CreateContainerResult, CreateDirectoryRequest,
        FileContentResponse, FileEntry, FileListResponse, FileMutationResponse, WriteFileRequest,
    },
    state::StateStore,
};

#[derive(Clone)]
pub struct ContainerLifecycleManager {
    config: Arc<Config>,
    docker: DockerClient,
    state_store: Arc<StateStore>,
}

impl ContainerLifecycleManager {
    pub fn new(config: Arc<Config>, docker: DockerClient, state_store: Arc<StateStore>) -> Self {
        Self {
            config,
            docker,
            state_store,
        }
    }

    pub async fn reconcile_on_startup(&self) -> Result<(), AppError> {
        let snapshot = self.state_store.snapshot().await;
        let expected_container_ids = snapshot.values().cloned().collect::<HashSet<_>>();
        let discovered = self
            .docker
            .reconcile_managed_containers(&expected_container_ids)
            .await?;

        for (server_id, container_id) in snapshot {
            if !discovered.contains(&container_id) {
                info!(%server_id, %container_id, "dropping stale state entry");
                self.state_store.remove(&server_id).await?;
            }
        }

        Ok(())
    }

    pub async fn create_backend_container(
        &self,
        request: BackendCreateContainerRequest,
    ) -> Result<CreateContainerResult, AppError> {
        request.validate()?;
        let server_id = request.server_id.to_string();
        self.state_store.ensure_unmapped(&server_id).await?;
        self.ensure_server_volume(&server_id, &request.game_kind)
            .await?;
        let created = self
            .docker
            .create_backend_container(&self.config, &request)
            .await?;
        self.state_store
            .insert(server_id, created.container_id.clone())
            .await?;

        Ok(created)
    }

    async fn ensure_server_volume(&self, server_id: &str, game_kind: &str) -> Result<(), AppError> {
        let server_path = self.config.base_volume_dir.join(server_id);
        fs::create_dir_all(&server_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to prepare server volume"))?;

        if game_kind == "rust" {
            for child in ["logs", "steamcmd", "serverfiles", "carbon", "oxide"] {
                fs::create_dir_all(server_path.join(child))
                    .await
                    .map_err(|error| {
                        map_fs_error(error, "failed to prepare rust server directories")
                    })?;
            }
        }

        #[cfg(unix)]
        {
            apply_world_writable_permissions(&server_path).await?;
        }

        Ok(())
    }

    pub async fn start_container(&self, request: ContainerActionRequest) -> Result<(), AppError> {
        request.validate()?;
        let container_id = self.resolve_owned_container_id(request).await?;
        self.docker.start_container(&container_id).await
    }

    pub async fn stop_container(&self, request: ContainerActionRequest) -> Result<(), AppError> {
        request.validate()?;
        let container_id = self.resolve_owned_container_id(request).await?;
        self.docker.stop_container(&container_id).await
    }

    pub async fn restart_container(&self, request: ContainerActionRequest) -> Result<(), AppError> {
        request.validate()?;
        let container_id = self.resolve_owned_container_id(request).await?;
        self.docker.restart_container(&container_id).await
    }

    pub async fn delete_container(&self, request: ContainerActionRequest) -> Result<(), AppError> {
        request.validate()?;
        let server_id = if let Some(server_id) = request.server_id.clone() {
            server_id
        } else {
            self.server_id_for_container(request.container_id.as_deref().ok_or_else(|| {
                AppError::new(
                    StatusCode::BAD_REQUEST,
                    "server_id or container_id is required",
                )
            })?)
            .await?
        };
        let container_id = self.resolve_owned_container_id(request).await?;
        self.docker.delete_container(&container_id).await?;
        self.state_store.remove(&server_id).await?;
        Ok(())
    }

    pub async fn status(&self, identifier: &str) -> Result<ContainerStatusResponse, AppError> {
        let container_id = self.lookup_owned_container(identifier).await?;
        let server_id = self.server_id_for_container(&container_id).await?;
        self.docker.fetch_status(&server_id, &container_id).await
    }

    pub async fn logs(
        &self,
        identifier: &str,
        tail: usize,
    ) -> Result<ContainerLogsResponse, AppError> {
        let container_id = self.lookup_owned_container(identifier).await?;
        let server_id = self.server_id_for_container(&container_id).await?;
        let logs = self.docker.fetch_logs(&container_id, tail).await?;

        Ok(ContainerLogsResponse {
            server_id,
            container_id,
            lines: logs,
        })
    }

    pub async fn list_files(
        &self,
        identifier: &str,
        path: Option<String>,
    ) -> Result<FileListResponse, AppError> {
        let server_id = self.resolve_server_id(identifier).await?;
        let requested_path = normalize_relative_path(path.as_deref().unwrap_or(""))?;
        let absolute_path = self.resolve_server_path(&server_id, &requested_path)?;

        let metadata = fs::metadata(&absolute_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to inspect directory"))?;
        if !metadata.is_dir() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "requested path is not a directory",
            ));
        }

        let mut entries = Vec::new();
        let mut reader = fs::read_dir(&absolute_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to read directory"))?;

        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|error| map_fs_error(error, "failed to read directory entry"))?
        {
            let metadata = entry
                .metadata()
                .await
                .map_err(|error| map_fs_error(error, "failed to read entry metadata"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let entry_path = if requested_path.is_empty() {
                name.clone()
            } else {
                format!("{requested_path}/{name}")
            };

            entries.push(FileEntry {
                name,
                path: entry_path,
                is_directory: metadata.is_dir(),
                size_bytes: if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
            });
        }

        entries.sort_by(|left, right| {
            right
                .is_directory
                .cmp(&left.is_directory)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(FileListResponse {
            server_id,
            path: requested_path,
            entries,
        })
    }

    pub async fn read_file(
        &self,
        identifier: &str,
        path: String,
    ) -> Result<FileContentResponse, AppError> {
        let server_id = self.resolve_server_id(identifier).await?;
        let requested_path = normalize_relative_path(&path)?;
        if requested_path.is_empty() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "path must point to a file",
            ));
        }

        let absolute_path = self.resolve_server_path(&server_id, &requested_path)?;
        let metadata = fs::metadata(&absolute_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to inspect file"))?;
        if !metadata.is_file() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "requested path is not a file",
            ));
        }

        let bytes = fs::read(&absolute_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to read file"))?;
        let content = String::from_utf8(bytes).map_err(|_| {
            AppError::new(
                StatusCode::BAD_REQUEST,
                "only UTF-8 text files can be viewed in the file explorer",
            )
        })?;

        Ok(FileContentResponse {
            server_id,
            path: requested_path,
            content,
        })
    }

    pub async fn write_file(
        &self,
        identifier: &str,
        request: WriteFileRequest,
    ) -> Result<FileMutationResponse, AppError> {
        let server_id = self.resolve_server_id(identifier).await?;
        let requested_path = normalize_relative_path(&request.path)?;
        if requested_path.is_empty() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "path must point to a file",
            ));
        }

        let absolute_path = self.resolve_server_path(&server_id, &requested_path)?;
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| map_fs_error(error, "failed to create parent directory"))?;
        }

        let bytes = if let Some(content_base64) = request.content_base64 {
            STANDARD.decode(content_base64).map_err(|_| {
                AppError::new(StatusCode::BAD_REQUEST, "invalid base64 file content")
            })?
        } else {
            request.content.into_bytes()
        };

        fs::write(&absolute_path, bytes)
            .await
            .map_err(|error| map_fs_error(error, "failed to write file"))?;

        Ok(FileMutationResponse {
            server_id,
            path: requested_path,
        })
    }

    pub async fn create_directory(
        &self,
        identifier: &str,
        request: CreateDirectoryRequest,
    ) -> Result<FileMutationResponse, AppError> {
        let server_id = self.resolve_server_id(identifier).await?;
        let requested_path = normalize_relative_path(&request.path)?;
        if requested_path.is_empty() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "path must point to a directory",
            ));
        }

        let absolute_path = self.resolve_server_path(&server_id, &requested_path)?;
        fs::create_dir_all(&absolute_path)
            .await
            .map_err(|error| map_fs_error(error, "failed to create directory"))?;

        Ok(FileMutationResponse {
            server_id,
            path: requested_path,
        })
    }

    async fn resolve_owned_container_id(
        &self,
        request: ContainerActionRequest,
    ) -> Result<String, AppError> {
        if let Some(server_id) = request.server_id {
            return self.lookup_owned_container(&server_id).await;
        }

        if let Some(container_id) = request.container_id {
            self.ensure_container_is_owned(&container_id).await?;
            return Ok(container_id);
        }

        Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "server_id or container_id is required",
        ))
    }

    async fn lookup_owned_container(&self, identifier: &str) -> Result<String, AppError> {
        if let Some(container_id) = self.state_store.get(identifier).await {
            self.ensure_container_is_owned(&container_id).await?;
            return Ok(container_id);
        }

        self.ensure_container_is_owned(identifier).await?;
        Ok(identifier.to_string())
    }

    async fn ensure_container_is_owned(&self, container_id: &str) -> Result<(), AppError> {
        let inspect = self.docker.inspect_container(container_id).await?;
        let labels = inspect
            .config
            .and_then(|config| config.labels)
            .unwrap_or_default();

        if !labels
            .get("game-host-daemon.server_id")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err(AppError::new(
                StatusCode::FORBIDDEN,
                "container ownership validation failed",
            ));
        }

        Ok(())
    }

    async fn server_id_for_container(&self, container_id: &str) -> Result<String, AppError> {
        let inspect = self.docker.inspect_container(&container_id).await?;
        let labels = inspect
            .config
            .and_then(|config| config.labels)
            .unwrap_or_default();

        labels
            .get("game-host-daemon.server_id")
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    StatusCode::FORBIDDEN,
                    "container ownership validation failed",
                )
            })
    }

    async fn resolve_server_id(&self, identifier: &str) -> Result<String, AppError> {
        if self.state_store.get(identifier).await.is_some() {
            return Ok(identifier.to_string());
        }

        if Uuid::parse_str(identifier).is_ok() {
            return Ok(identifier.to_string());
        }

        self.server_id_for_container(identifier).await
    }

    fn resolve_server_path(
        &self,
        server_id: &str,
        relative_path: &str,
    ) -> Result<PathBuf, AppError> {
        let base = self.config.base_volume_dir.join(server_id);
        let target = if relative_path.is_empty() {
            base
        } else {
            base.join(relative_path)
        };

        Ok(target)
    }
}

fn normalize_relative_path(path: &str) -> Result<String, AppError> {
    let raw = path.trim().replace('\\', "/");
    if raw.is_empty() || raw == "." {
        return Ok(String::new());
    }

    let candidate = Path::new(&raw);
    if candidate.is_absolute() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "absolute paths are not allowed",
        ));
    }

    let mut parts = Vec::new();
    for component in candidate.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    "path traversal is not allowed",
                ))
            }
            _ => {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    "invalid path component",
                ))
            }
        }
    }

    Ok(parts.join("/"))
}

fn map_fs_error(error: std::io::Error, context: &str) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => AppError::new(StatusCode::NOT_FOUND, "resource not found"),
        std::io::ErrorKind::PermissionDenied => {
            AppError::new(StatusCode::FORBIDDEN, "permission denied")
        }
        _ => AppError::internal(format!("{context}: {error}")),
    }
}

#[cfg(unix)]
async fn apply_world_writable_permissions(path: &Path) -> Result<(), AppError> {
    let mut stack = vec![path.to_path_buf()];

    while let Some(current) = stack.pop() {
        let metadata = fs::metadata(&current)
            .await
            .map_err(|error| map_fs_error(error, "failed to inspect server volume path"))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(if metadata.is_dir() { 0o777 } else { 0o666 });
        fs::set_permissions(&current, permissions)
            .await
            .map_err(|error| map_fs_error(error, "failed to update server volume permissions"))?;

        if metadata.is_dir() {
            let mut reader = fs::read_dir(&current)
                .await
                .map_err(|error| map_fs_error(error, "failed to read server volume directory"))?;
            while let Some(entry) = reader
                .next_entry()
                .await
                .map_err(|error| map_fs_error(error, "failed to read server volume entry"))?
            {
                stack.push(entry.path());
            }
        }
    }

    Ok(())
}

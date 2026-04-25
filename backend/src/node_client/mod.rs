use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::errors::AppError;

#[derive(Clone)]
pub struct NodeDaemonClient {
    client: reqwest::Client,
}

impl NodeDaemonClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_container(
        &self,
        node_url: &str,
        api_token: &str,
        request: &CreateContainerRequest,
    ) -> Result<CreateContainerResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/create"))
                .bearer_auth(api_token)
                .json(request),
        )
        .await
    }

    pub async fn start_container(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeStatusResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/start"))
                .bearer_auth(api_token)
                .json(&ContainerActionRequest {
                    container_id: container_id.to_owned(),
                }),
        )
        .await
    }

    pub async fn stop_container(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeStatusResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/stop"))
                .bearer_auth(api_token)
                .json(&ContainerActionRequest {
                    container_id: container_id.to_owned(),
                }),
        )
        .await
    }

    pub async fn restart_container(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeStatusResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/restart"))
                .bearer_auth(api_token)
                .json(&ContainerActionRequest {
                    container_id: container_id.to_owned(),
                }),
        )
        .await
    }

    pub async fn delete_container(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeStatusResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/delete"))
                .bearer_auth(api_token)
                .json(&ContainerActionRequest {
                    container_id: container_id.to_owned(),
                }),
        )
        .await
    }

    pub async fn get_container_status(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeStatusResponse, AppError> {
        self.send_json(
            self.client
                .get(format!("{node_url}/containers/{container_id}/status"))
                .bearer_auth(api_token),
        )
        .await
    }

    pub async fn get_container_logs(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
    ) -> Result<NodeLogsResponse, AppError> {
        self.send_json(
            self.client
                .get(format!("{node_url}/containers/{container_id}/logs"))
                .bearer_auth(api_token),
        )
        .await
    }

    pub async fn list_files(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
        path: Option<&str>,
    ) -> Result<NodeFileListResponse, AppError> {
        let mut request = self
            .client
            .get(format!("{node_url}/containers/{container_id}/files"))
            .bearer_auth(api_token);

        if let Some(path) = path {
            request = request.query(&[("path", path)]);
        }

        self.send_json(request).await
    }

    pub async fn read_file(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
        path: &str,
    ) -> Result<NodeFileContentResponse, AppError> {
        self.send_json(
            self.client
                .get(format!("{node_url}/containers/{container_id}/file"))
                .bearer_auth(api_token)
                .query(&[("path", path)]),
        )
        .await
    }

    pub async fn write_file(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
        path: &str,
        content: &str,
        content_base64: Option<&str>,
    ) -> Result<NodeFileMutationResponse, AppError> {
        self.send_json(
            self.client
                .put(format!("{node_url}/containers/{container_id}/file"))
                .bearer_auth(api_token)
                .json(&NodeWriteFileRequest {
                    path: path.to_owned(),
                    content: content.to_owned(),
                    content_base64: content_base64.map(ToOwned::to_owned),
                }),
        )
        .await
    }

    pub async fn create_directory(
        &self,
        node_url: &str,
        api_token: &str,
        container_id: &str,
        path: &str,
    ) -> Result<NodeFileMutationResponse, AppError> {
        self.send_json(
            self.client
                .post(format!("{node_url}/containers/{container_id}/directories"))
                .bearer_auth(api_token)
                .json(&NodeCreateDirectoryRequest {
                    path: path.to_owned(),
                }),
        )
        .await
    }

    async fn send_json<T>(&self, request: reqwest::RequestBuilder) -> Result<T, AppError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = request.send().await.map_err(AppError::NodeRequest)?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::NodeDaemon {
                status,
                message: body,
            });
        }

        response.json::<T>().await.map_err(AppError::NodeRequest)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateContainerRequest {
    pub server_id: uuid::Uuid,
    pub name: String,
    pub game_kind: String,
    pub server_settings: Value,
    pub allocated_port: i32,
    pub memory_limit_mb: i32,
    pub cpu_limit_percent: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateContainerResponse {
    pub container_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
struct ContainerActionRequest {
    pub container_id: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeStatusResponse {
    #[serde(alias = "state")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeLogsResponse {
    pub lines: Vec<NodeLogLine>,
}

#[derive(Debug, Deserialize)]
pub struct NodeLogLine {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub line: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeFileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct NodeFileListResponse {
    pub server_id: String,
    pub path: String,
    pub entries: Vec<NodeFileEntry>,
}

#[derive(Debug, Deserialize)]
pub struct NodeFileContentResponse {
    pub server_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeFileMutationResponse {
    pub server_id: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
struct NodeWriteFileRequest {
    pub path: String,
    pub content: String,
    pub content_base64: Option<String>,
}

#[derive(Debug, Serialize)]
struct NodeCreateDirectoryRequest {
    pub path: String,
}

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

use crate::{config::Config, error::AppError, manager::ContainerLifecycleManager};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub manager: Arc<ContainerLifecycleManager>,
}

pub struct StateStore {
    path: PathBuf,
    inner: RwLock<HashMap<String, String>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedState {
    containers: HashMap<String, String>,
}

impl StateStore {
    pub async fn load(path: PathBuf) -> Result<Self, AppError> {
        let containers = match fs::read(&path).await {
            Ok(bytes) => {
                serde_json::from_slice::<PersistedState>(&bytes)
                    .map_err(|error| {
                        AppError::internal(format!("failed to parse state file: {error}"))
                    })?
                    .containers
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => {
                return Err(AppError::internal(format!(
                    "failed to read state file: {error}"
                )))
            }
        };

        Ok(Self {
            path,
            inner: RwLock::new(containers),
        })
    }

    pub async fn snapshot(&self) -> HashMap<String, String> {
        self.inner.read().await.clone()
    }

    pub async fn get(&self, server_id: &str) -> Option<String> {
        self.inner.read().await.get(server_id).cloned()
    }

    pub async fn ensure_unmapped(&self, server_id: &str) -> Result<(), AppError> {
        if self.inner.read().await.contains_key(server_id) {
            return Err(AppError::new(
                StatusCode::CONFLICT,
                "server already has a managed container",
            ));
        }

        Ok(())
    }

    pub async fn insert(&self, server_id: String, container_id: String) -> Result<(), AppError> {
        {
            let mut state = self.inner.write().await;
            state.insert(server_id, container_id);
        }
        self.persist().await
    }

    pub async fn remove(&self, server_id: &str) -> Result<(), AppError> {
        {
            let mut state = self.inner.write().await;
            state.remove(server_id);
        }
        self.persist().await
    }

    async fn persist(&self) -> Result<(), AppError> {
        let snapshot = PersistedState {
            containers: self.inner.read().await.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| AppError::internal(format!("failed to serialize state: {error}")))?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await.map_err(|error| {
                AppError::internal(format!("failed to create state directory: {error}"))
            })?;
        }

        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, bytes)
            .await
            .map_err(|error| AppError::internal(format!("failed to write state file: {error}")))?;
        fs::rename(&temp_path, &self.path).await.map_err(|error| {
            AppError::internal(format!("failed to finalize state file: {error}"))
        })?;

        Ok(())
    }
}

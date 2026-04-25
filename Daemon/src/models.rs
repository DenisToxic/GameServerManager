use std::collections::HashMap;

use chrono::{DateTime, Utc};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ContainerActionRequest {
    pub server_id: Option<String>,
    pub container_id: Option<String>,
}

impl ContainerActionRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self
            .server_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
            && self
                .container_id
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "server_id or container_id is required",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct BackendCreateContainerRequest {
    pub server_id: Uuid,
    pub name: String,
    pub game_kind: String,
    #[serde(default)]
    pub server_settings: Value,
    pub allocated_port: i32,
    pub memory_limit_mb: i32,
    pub cpu_limit_percent: i32,
}

impl BackendCreateContainerRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.trim().is_empty() {
            return Err(AppError::new(StatusCode::BAD_REQUEST, "name is required"));
        }

        if !matches!(self.game_kind.as_str(), "minecraft" | "rust" | "hytale") {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "game_kind must be one of: minecraft, rust, hytale",
            ));
        }

        if !(1..=65535).contains(&self.allocated_port) {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "allocated_port must be between 1 and 65535",
            ));
        }

        if self.memory_limit_mb <= 0 || self.cpu_limit_percent <= 0 {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "memory_limit_mb and cpu_limit_percent must be greater than zero",
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ContainerSpec {
    pub server_id: String,
    pub image: String,
    pub environment: HashMap<String, String>,
    pub ports: Vec<ContainerPortMapping>,
    pub memory_limit_mb: Option<i64>,
    pub cpu_limit: Option<f64>,
    pub working_dir: String,
    pub volume_host_path: String,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
    pub container_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContainerPortMapping {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String,
    pub host_ip: String,
}

impl ContainerPortMapping {
    pub fn key(&self) -> String {
        format!("{}/{}", self.container_port, self.protocol)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateContainerResult {
    pub server_id: String,
    pub container_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ApiMessageResponse {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ContainerStatusResponse {
    pub server_id: String,
    pub container_id: String,
    pub state: String,
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ContainerLogsResponse {
    pub server_id: String,
    pub container_id: String,
    pub lines: Vec<ContainerLogLine>,
}

#[derive(Debug, Serialize)]
pub struct ContainerLogLine {
    pub timestamp: DateTime<Utc>,
    pub line: String,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct FilePathQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub server_id: String,
    pub path: String,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub server_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub content_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDirectoryRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FileMutationResponse {
    pub server_id: String,
    pub path: String,
}

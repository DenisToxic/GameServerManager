use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

pub const GAME_KIND_MINECRAFT: &str = "minecraft";
pub const GAME_KIND_RUST: &str = "rust";
pub const GAME_KIND_HYTALE: &str = "hytale";

pub fn is_supported_game_kind(value: &str) -> bool {
    matches!(
        value,
        GAME_KIND_MINECRAFT | GAME_KIND_RUST | GAME_KIND_HYTALE
    )
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServerRequest {
    #[validate(length(min = 3, max = 64))]
    pub name: String,
    pub node_id: i32,
    pub game_kind: String,
    #[serde(default)]
    pub server_settings: Value,
    #[validate(range(min = 1, max = 65535))]
    pub allocated_port: i32,
    #[validate(range(min = 128, max = 262144))]
    pub memory_limit_mb: i32,
    #[validate(range(min = 1, max = 1000))]
    pub cpu_limit_percent: i32,
}

#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub node_id: i32,
    pub game_kind: String,
    pub server_settings: Value,
    pub name: String,
    pub status: String,
    pub docker_container_id: Option<String>,
    pub allocated_port: i32,
    pub memory_limit_mb: i32,
    pub cpu_limit_percent: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ServerStatusResponse {
    pub server_id: Uuid,
    pub desired_status: String,
    pub actual_status: String,
}

#[derive(Debug, Serialize)]
pub struct ServerLogsResponse {
    pub server_id: Uuid,
    pub lines: Vec<ServerLogLine>,
}

#[derive(Debug, Serialize)]
pub struct ServerLogLine {
    pub timestamp: DateTime<Utc>,
    pub line: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerFilePathQuery {
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WriteServerFileRequest {
    pub path: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub content_base64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServerDirectoryRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerFileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerFileListResponse {
    pub server_id: String,
    pub path: String,
    pub entries: Vec<ServerFileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerFileContentResponse {
    pub server_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerFileMutationResponse {
    pub server_id: String,
    pub path: String,
}

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct NodeRecord {
    pub id: i32,
    pub name: String,
    pub api_url: String,
    pub api_token: String,
}

#[derive(Debug, FromRow)]
pub struct ServerRecord {
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

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct ServerLogRecord {
    pub id: i64,
    pub server_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub line: String,
}

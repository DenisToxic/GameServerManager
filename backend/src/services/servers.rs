use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        api::servers::{
            is_supported_game_kind, CreateServerDirectoryRequest, CreateServerRequest,
            ServerFileContentResponse, ServerFileEntry, ServerFileListResponse,
            ServerFileMutationResponse, ServerLogLine, ServerLogsResponse, ServerResponse,
            ServerStatusResponse, WriteServerFileRequest,
        },
        db::{NodeRecord, ServerLogRecord, ServerRecord},
    },
    node_client::{CreateContainerRequest, NodeDaemonClient},
};

use super::errors::AppError;

#[derive(Clone)]
pub struct ServerService {
    node_client: NodeDaemonClient,
}

impl ServerService {
    pub fn new(node_client: NodeDaemonClient) -> Self {
        Self { node_client }
    }

    pub async fn list_servers(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ServerResponse>, AppError> {
        let servers = sqlx::query_as::<_, ServerRecord>(
            r#"
            SELECT
                id,
                user_id,
                node_id,
                game_kind,
                server_settings,
                name,
                status,
                docker_container_id,
                allocated_port,
                memory_limit_mb,
                cpu_limit_percent,
                created_at
            FROM servers
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(servers.into_iter().map(map_server).collect())
    }

    pub async fn get_server_details(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerResponse, AppError> {
        let server = self.get_server(pool, user_id, server_id).await?;
        Ok(map_server(server))
    }

    pub async fn create_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: CreateServerRequest,
    ) -> Result<ServerResponse, AppError> {
        payload
            .validate()
            .map_err(|error| AppError::Validation(error.to_string()))?;

        if !is_supported_game_kind(&payload.game_kind) {
            return Err(AppError::Validation(
                "game_kind must be one of: minecraft, rust, hytale".to_owned(),
            ));
        }

        if !payload.server_settings.is_object() {
            return Err(AppError::Validation(
                "server_settings must be a JSON object".to_owned(),
            ));
        }

        let server_id = Uuid::new_v4();
        let node = self.get_node(pool, payload.node_id).await?;

        let existing_port: Option<Uuid> =
            sqlx::query_scalar(r#"SELECT id FROM servers WHERE allocated_port = $1"#)
                .bind(payload.allocated_port)
                .fetch_optional(pool)
                .await?;

        if existing_port.is_some() {
            return Err(AppError::Conflict(
                "allocated port is already in use".to_owned(),
            ));
        }

        let initial_server = sqlx::query_as::<_, ServerRecord>(
            r#"
            INSERT INTO servers (
                id,
                user_id,
                node_id,
                game_kind,
                server_settings,
                name,
                status,
                docker_container_id,
                allocated_port,
                memory_limit_mb,
                cpu_limit_percent
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'stopped', NULL, $7, $8, $9)
            RETURNING
                id,
                user_id,
                node_id,
                game_kind,
                server_settings,
                name,
                status,
                docker_container_id,
                allocated_port,
                memory_limit_mb,
                cpu_limit_percent,
                created_at
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .bind(payload.node_id)
        .bind(&payload.game_kind)
        .bind(&payload.server_settings)
        .bind(payload.name)
        .bind(payload.allocated_port)
        .bind(payload.memory_limit_mb)
        .bind(payload.cpu_limit_percent)
        .fetch_one(pool)
        .await?;

        let container_result = self
            .node_client
            .create_container(
                &node.api_url,
                &node.api_token,
                &CreateContainerRequest {
                    server_id,
                    name: initial_server.name.clone(),
                    game_kind: initial_server.game_kind.clone(),
                    server_settings: initial_server.server_settings.clone(),
                    allocated_port: initial_server.allocated_port,
                    memory_limit_mb: initial_server.memory_limit_mb,
                    cpu_limit_percent: initial_server.cpu_limit_percent,
                },
            )
            .await;

        let container = match container_result {
            Ok(container) => container,
            Err(error) => {
                sqlx::query(
                    r#"
                    UPDATE servers
                    SET status = 'error'
                    WHERE id = $1
                    "#,
                )
                .bind(server_id)
                .execute(pool)
                .await?;

                return Err(error);
            }
        };

        let server = sqlx::query_as::<_, ServerRecord>(
            r#"
            UPDATE servers
            SET status = $2, docker_container_id = $3
            WHERE id = $1
            RETURNING
                id,
                user_id,
                node_id,
                game_kind,
                server_settings,
                name,
                status,
                docker_container_id,
                allocated_port,
                memory_limit_mb,
                cpu_limit_percent,
                created_at
            "#,
        )
        .bind(server_id)
        .bind(container.status)
        .bind(Some(container.container_id))
        .fetch_one(pool)
        .await?;

        Ok(map_server(server))
    }

    pub async fn start_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerStatusResponse, AppError> {
        self.transition_server(pool, user_id, server_id, "starting", NodeAction::Start)
            .await
    }

    pub async fn stop_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerStatusResponse, AppError> {
        self.transition_server(pool, user_id, server_id, "stopping", NodeAction::Stop)
            .await
    }

    pub async fn restart_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerStatusResponse, AppError> {
        self.transition_server(pool, user_id, server_id, "starting", NodeAction::Restart)
            .await
    }

    pub async fn get_server_status(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerStatusResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;
        let container_id = server.docker_container_id.ok_or(AppError::Conflict(
            "server has no assigned container id".to_owned(),
        ))?;

        let status = self
            .node_client
            .get_container_status(&node.api_url, &node.api_token, &container_id)
            .await?;

        Ok(ServerStatusResponse {
            server_id: server.id,
            desired_status: server.status,
            actual_status: status.status,
        })
    }

    pub async fn get_server_logs(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerLogsResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;
        let container_id = server.docker_container_id.ok_or(AppError::Conflict(
            "server has no assigned container id".to_owned(),
        ))?;

        let node_logs = self
            .node_client
            .get_container_logs(&node.api_url, &node.api_token, &container_id)
            .await?;

        for entry in &node_logs.lines {
            sqlx::query(
                r#"
                INSERT INTO server_logs (server_id, timestamp, line)
                VALUES ($1, $2, $3)
                ON CONFLICT (server_id, timestamp, line) DO NOTHING
                "#,
            )
            .bind(server.id)
            .bind(entry.timestamp)
            .bind(&entry.line)
            .execute(pool)
            .await?;
        }

        let stored_logs = sqlx::query_as::<_, ServerLogRecord>(
            r#"
            SELECT id, server_id, timestamp, line
            FROM server_logs
            WHERE server_id = $1
            ORDER BY timestamp DESC
            LIMIT 200
            "#,
        )
        .bind(server.id)
        .fetch_all(pool)
        .await?;

        Ok(ServerLogsResponse {
            server_id: server.id,
            lines: stored_logs
                .into_iter()
                .map(|row| ServerLogLine {
                    timestamp: row.timestamp,
                    line: row.line,
                })
                .collect(),
        })
    }

    pub async fn delete_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<(), AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;

        if let Some(container_id) = server.docker_container_id.clone() {
            self.node_client
                .delete_container(&node.api_url, &node.api_token, &container_id)
                .await?;
        }

        sqlx::query(
            r#"
            DELETE FROM servers
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(server.id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list_server_files(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        path: Option<String>,
    ) -> Result<ServerFileListResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;

        let response = self
            .node_client
            .list_files(
                &node.api_url,
                &node.api_token,
                &server.id.to_string(),
                path.as_deref(),
            )
            .await?;

        Ok(ServerFileListResponse {
            server_id: response.server_id,
            path: response.path,
            entries: response
                .entries
                .into_iter()
                .map(|entry| ServerFileEntry {
                    name: entry.name,
                    path: entry.path,
                    is_directory: entry.is_directory,
                    size_bytes: entry.size_bytes,
                })
                .collect(),
        })
    }

    pub async fn read_server_file(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        path: String,
    ) -> Result<ServerFileContentResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;

        let response = self
            .node_client
            .read_file(
                &node.api_url,
                &node.api_token,
                &server.id.to_string(),
                &path,
            )
            .await?;

        Ok(ServerFileContentResponse {
            server_id: response.server_id,
            path: response.path,
            content: response.content,
        })
    }

    pub async fn write_server_file(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        payload: WriteServerFileRequest,
    ) -> Result<ServerFileMutationResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;

        let response = self
            .node_client
            .write_file(
                &node.api_url,
                &node.api_token,
                &server.id.to_string(),
                &payload.path,
                &payload.content,
                payload.content_base64.as_deref(),
            )
            .await?;

        Ok(ServerFileMutationResponse {
            server_id: response.server_id,
            path: response.path,
        })
    }

    pub async fn create_server_directory(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        payload: CreateServerDirectoryRequest,
    ) -> Result<ServerFileMutationResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;

        let response = self
            .node_client
            .create_directory(
                &node.api_url,
                &node.api_token,
                &server.id.to_string(),
                &payload.path,
            )
            .await?;

        Ok(ServerFileMutationResponse {
            server_id: response.server_id,
            path: response.path,
        })
    }

    async fn transition_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        desired_status: &str,
        action: NodeAction,
    ) -> Result<ServerStatusResponse, AppError> {
        let (server, node) = self.get_server_and_node(pool, user_id, server_id).await?;
        let container_id = match (server.docker_container_id.clone(), &action) {
            (Some(container_id), _) => container_id,
            (None, NodeAction::Start | NodeAction::Restart) => {
                self.provision_container(pool, &server, &node).await?
            }
            (None, NodeAction::Stop) => {
                return Err(AppError::Conflict(
                    "server has no assigned container id".to_owned(),
                ))
            }
        };

        sqlx::query(
            r#"
            UPDATE servers
            SET status = $1
            WHERE id = $2
            "#,
        )
        .bind(desired_status)
        .bind(server.id)
        .execute(pool)
        .await?;

        let node_status = match action {
            NodeAction::Start => {
                self.node_client
                    .start_container(&node.api_url, &node.api_token, &container_id)
                    .await
            }
            NodeAction::Stop => {
                self.node_client
                    .stop_container(&node.api_url, &node.api_token, &container_id)
                    .await
            }
            NodeAction::Restart => {
                self.node_client
                    .restart_container(&node.api_url, &node.api_token, &container_id)
                    .await
            }
        };

        let node_status = match node_status {
            Ok(status) => status,
            Err(error) => {
                sqlx::query(
                    r#"
                    UPDATE servers
                    SET status = 'error'
                    WHERE id = $1
                    "#,
                )
                .bind(server.id)
                .execute(pool)
                .await?;

                return Err(error);
            }
        };

        sqlx::query(
            r#"
            UPDATE servers
            SET status = $1
            WHERE id = $2
            "#,
        )
        .bind(node_status.status.clone())
        .bind(server.id)
        .execute(pool)
        .await?;

        Ok(ServerStatusResponse {
            server_id: server.id,
            desired_status: desired_status.to_owned(),
            actual_status: node_status.status,
        })
    }

    async fn provision_container(
        &self,
        pool: &PgPool,
        server: &ServerRecord,
        node: &NodeRecord,
    ) -> Result<String, AppError> {
        let container = self
            .node_client
            .create_container(
                &node.api_url,
                &node.api_token,
                &CreateContainerRequest {
                    server_id: server.id,
                    name: server.name.clone(),
                    game_kind: server.game_kind.clone(),
                    server_settings: server.server_settings.clone(),
                    allocated_port: server.allocated_port,
                    memory_limit_mb: server.memory_limit_mb,
                    cpu_limit_percent: server.cpu_limit_percent,
                },
            )
            .await?;

        sqlx::query(
            r#"
            UPDATE servers
            SET docker_container_id = $2, status = $3
            WHERE id = $1
            "#,
        )
        .bind(server.id)
        .bind(&container.container_id)
        .bind(&container.status)
        .execute(pool)
        .await?;

        Ok(container.container_id)
    }

    async fn get_server_and_node(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<(ServerRecord, NodeRecord), AppError> {
        let server = self.get_server(pool, user_id, server_id).await?;
        let node = self.get_node(pool, server.node_id).await?;
        Ok((server, node))
    }

    async fn get_server(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> Result<ServerRecord, AppError> {
        sqlx::query_as::<_, ServerRecord>(
            r#"
            SELECT
                id,
                user_id,
                node_id,
                game_kind,
                server_settings,
                name,
                status,
                docker_container_id,
                allocated_port,
                memory_limit_mb,
                cpu_limit_percent,
                created_at
            FROM servers
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    async fn get_node(&self, pool: &PgPool, node_id: i32) -> Result<NodeRecord, AppError> {
        sqlx::query_as::<_, NodeRecord>(
            r#"
            SELECT id, name, api_url, api_token
            FROM nodes
            WHERE id = $1
            "#,
        )
        .bind(node_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
    }
}

fn map_server(server: ServerRecord) -> ServerResponse {
    ServerResponse {
        id: server.id,
        user_id: server.user_id,
        node_id: server.node_id,
        game_kind: server.game_kind,
        server_settings: server.server_settings,
        name: server.name,
        status: server.status,
        docker_container_id: server.docker_container_id,
        allocated_port: server.allocated_port,
        memory_limit_mb: server.memory_limit_mb,
        cpu_limit_percent: server.cpu_limit_percent,
        created_at: server.created_at,
    }
}

enum NodeAction {
    Start,
    Stop,
    Restart,
}

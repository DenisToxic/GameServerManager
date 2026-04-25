use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::AppConfig,
    services::{auth::AuthService, servers::ServerService},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub auth_service: AuthService,
    pub server_service: ServerService,
}

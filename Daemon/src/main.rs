mod auth;
mod config;
mod docker;
mod error;
mod game_presets;
mod manager;
mod models;
mod routes;
mod state;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    config::Config,
    docker::DockerClient,
    error::AppError,
    manager::ContainerLifecycleManager,
    state::{AppState, StateStore},
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,game_host_daemon=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env()?);
    let state_store = Arc::new(StateStore::load(config.state_file.clone()).await?);
    let docker = DockerClient::new(config.managed_label.clone())?;
    let manager = Arc::new(ContainerLifecycleManager::new(
        Arc::clone(&config),
        docker,
        state_store,
    ));

    manager.reconcile_on_startup().await?;

    let shared_state = Arc::new(AppState {
        config: Arc::clone(&config),
        manager,
    });

    let app = Router::new()
        .route("/containers/create", post(routes::create_container))
        .route("/containers/start", post(routes::start_container))
        .route("/containers/stop", post(routes::stop_container))
        .route("/containers/restart", post(routes::restart_container))
        .route("/containers/delete", post(routes::delete_container))
        .route("/containers/:id/status", get(routes::container_status))
        .route("/containers/:id/logs", get(routes::container_logs))
        .route("/containers/:id/files", get(routes::list_files))
        .route(
            "/containers/:id/file",
            get(routes::read_file).put(routes::write_file),
        )
        .route(
            "/containers/:id/directories",
            post(routes::create_directory),
        )
        .layer(middleware::from_fn_with_state(
            Arc::clone(&shared_state),
            auth::require_bearer_auth,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::clone(&shared_state));

    let address: SocketAddr = config
        .bind_addr
        .parse()
        .map_err(|error| AppError::internal(format!("invalid bind address: {error}")))?;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| AppError::internal(format!("failed to bind socket: {error}")))?;

    info!(address = %config.bind_addr, "daemon listening");
    axum::serve(listener, app)
        .await
        .map_err(|error| AppError::internal(format!("server error: {error}")))
}

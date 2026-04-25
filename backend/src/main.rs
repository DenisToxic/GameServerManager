mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod node_client;
mod services;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use config::AppConfig;
use db::AppState;
use handlers::{auth, servers};
use middleware::auth::require_auth;
use node_client::NodeDaemonClient;
use services::{auth::AuthService, servers::ServerService};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Arc::new(AppConfig::from_env()?);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("failed to connect to postgres")?;

    let state = AppState {
        pool,
        config: config.clone(),
        auth_service: AuthService::new(config.clone()),
        server_service: ServerService::new(NodeDaemonClient::new()),
    };

    let protected_routes = Router::new()
        .route(
            "/servers",
            get(servers::list_servers).post(servers::create_server),
        )
        .route(
            "/servers/:id",
            get(servers::get_server).delete(servers::delete_server),
        )
        .route("/servers/:id/start", post(servers::start_server))
        .route("/servers/:id/stop", post(servers::stop_server))
        .route("/servers/:id/restart", post(servers::restart_server))
        .route("/servers/:id/status", get(servers::get_server_status))
        .route("/servers/:id/logs", get(servers::get_server_logs))
        .route("/servers/:id/files", get(servers::list_server_files))
        .route(
            "/servers/:id/file",
            get(servers::read_server_file).put(servers::write_server_file),
        )
        .route(
            "/servers/:id/directories",
            post(servers::create_server_directory),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    let app = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .merge(protected_routes)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let address: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .context("invalid bind address")?;

    tracing::info!("listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("failed to bind tcp listener")?;

    axum::serve(listener, app)
        .await
        .context("server exited unexpectedly")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gsm_backend=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

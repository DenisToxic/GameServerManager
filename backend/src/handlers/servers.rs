use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::models::api::servers::{
    CreateServerDirectoryRequest, ServerFileContentResponse, ServerFileListResponse,
    ServerFileMutationResponse, ServerFilePathQuery, WriteServerFileRequest,
};
use crate::{
    db::AppState,
    middleware::auth::AuthenticatedUser,
    models::api::{
        servers::{CreateServerRequest, ServerLogsResponse, ServerResponse, ServerStatusResponse},
        ApiResponse,
    },
    services::errors::AppError,
};

pub async fn list_servers(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<Vec<ServerResponse>>>, AppError> {
    let servers = state
        .server_service
        .list_servers(&state.pool, user.user_id)
        .await?;
    Ok(Json(ApiResponse::new(servers)))
}

pub async fn create_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateServerRequest>,
) -> Result<Json<ApiResponse<ServerResponse>>, AppError> {
    let server = state
        .server_service
        .create_server(&state.pool, user.user_id, payload)
        .await?;
    Ok(Json(ApiResponse::new(server)))
}

pub async fn get_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerResponse>>, AppError> {
    let server = state
        .server_service
        .get_server_details(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(server)))
}

pub async fn start_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerStatusResponse>>, AppError> {
    let status = state
        .server_service
        .start_server(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(status)))
}

pub async fn stop_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerStatusResponse>>, AppError> {
    let status = state
        .server_service
        .stop_server(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(status)))
}

pub async fn restart_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerStatusResponse>>, AppError> {
    let status = state
        .server_service
        .restart_server(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(status)))
}

pub async fn get_server_status(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerStatusResponse>>, AppError> {
    let status = state
        .server_service
        .get_server_status(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(status)))
}

pub async fn get_server_logs(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ServerLogsResponse>>, AppError> {
    let logs = state
        .server_service
        .get_server_logs(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(logs)))
}

pub async fn delete_server(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    state
        .server_service
        .delete_server(&state.pool, user.user_id, id)
        .await?;
    Ok(Json(ApiResponse::new(serde_json::json!({
        "deleted": true,
        "server_id": id
    }))))
}

pub async fn list_server_files(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<ServerFilePathQuery>,
) -> Result<Json<ApiResponse<ServerFileListResponse>>, AppError> {
    let files = state
        .server_service
        .list_server_files(&state.pool, user.user_id, id, query.path)
        .await?;
    Ok(Json(ApiResponse::new(files)))
}

pub async fn read_server_file(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<ServerFilePathQuery>,
) -> Result<Json<ApiResponse<ServerFileContentResponse>>, AppError> {
    let file = state
        .server_service
        .read_server_file(
            &state.pool,
            user.user_id,
            id,
            query.path.unwrap_or_default(),
        )
        .await?;
    Ok(Json(ApiResponse::new(file)))
}

pub async fn write_server_file(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<WriteServerFileRequest>,
) -> Result<Json<ApiResponse<ServerFileMutationResponse>>, AppError> {
    let file = state
        .server_service
        .write_server_file(&state.pool, user.user_id, id, payload)
        .await?;
    Ok(Json(ApiResponse::new(file)))
}

pub async fn create_server_directory(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateServerDirectoryRequest>,
) -> Result<Json<ApiResponse<ServerFileMutationResponse>>, AppError> {
    let directory = state
        .server_service
        .create_server_directory(&state.pool, user.user_id, id, payload)
        .await?;
    Ok(Json(ApiResponse::new(directory)))
}

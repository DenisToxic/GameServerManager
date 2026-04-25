use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    error::AppError,
    models::{
        ApiMessageResponse, BackendCreateContainerRequest, ContainerActionRequest,
        ContainerLogsResponse, ContainerStatusResponse, CreateContainerResult,
        CreateDirectoryRequest, FileContentResponse, FileListResponse, FileMutationResponse,
        FilePathQuery, LogsQuery, WriteFileRequest,
    },
    state::AppState,
};

pub async fn create_container(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BackendCreateContainerRequest>,
) -> Result<(StatusCode, Json<CreateContainerResult>), AppError> {
    let response = state.manager.create_backend_container(payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn start_container(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContainerActionRequest>,
) -> Result<Json<ApiMessageResponse>, AppError> {
    state.manager.start_container(payload).await?;
    Ok(Json(ApiMessageResponse {
        status: "running".to_string(),
    }))
}

pub async fn stop_container(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContainerActionRequest>,
) -> Result<Json<ApiMessageResponse>, AppError> {
    state.manager.stop_container(payload).await?;
    Ok(Json(ApiMessageResponse {
        status: "stopped".to_string(),
    }))
}

pub async fn restart_container(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContainerActionRequest>,
) -> Result<Json<ApiMessageResponse>, AppError> {
    state.manager.restart_container(payload).await?;
    Ok(Json(ApiMessageResponse {
        status: "running".to_string(),
    }))
}

pub async fn delete_container(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ContainerActionRequest>,
) -> Result<Json<ApiMessageResponse>, AppError> {
    state.manager.delete_container(payload).await?;
    Ok(Json(ApiMessageResponse {
        status: "deleted".to_string(),
    }))
}

pub async fn container_status(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> Result<Json<ContainerStatusResponse>, AppError> {
    let response = state.manager.status(&server_id).await?;
    Ok(Json(response))
}

pub async fn container_logs(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<ContainerLogsResponse>, AppError> {
    let tail = query.tail.unwrap_or(200).min(10_000);
    let response = state.manager.logs(&server_id, tail).await?;
    Ok(Json(response))
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    Query(query): Query<FilePathQuery>,
) -> Result<Json<FileListResponse>, AppError> {
    let response = state.manager.list_files(&server_id, query.path).await?;
    Ok(Json(response))
}

pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    Query(query): Query<FilePathQuery>,
) -> Result<Json<FileContentResponse>, AppError> {
    let path = query.path.unwrap_or_default();
    let response = state.manager.read_file(&server_id, path).await?;
    Ok(Json(response))
}

pub async fn write_file(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    Json(payload): Json<WriteFileRequest>,
) -> Result<Json<FileMutationResponse>, AppError> {
    let response = state.manager.write_file(&server_id, payload).await?;
    Ok(Json(response))
}

pub async fn create_directory(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    Json(payload): Json<CreateDirectoryRequest>,
) -> Result<(StatusCode, Json<FileMutationResponse>), AppError> {
    let response = state.manager.create_directory(&server_id, payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

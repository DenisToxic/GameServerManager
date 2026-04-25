use axum::{extract::State, Json};

use crate::{
    db::AppState,
    models::api::{
        auth::{AuthResponse, LoginRequest, RegisterRequest},
        ApiResponse,
    },
    services::errors::AppError,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let auth = state.auth_service.register(&state.pool, payload).await?;
    Ok(Json(ApiResponse::new(auth)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let auth = state.auth_service.login(&state.pool, payload).await?;
    Ok(Json(ApiResponse::new(auth)))
}

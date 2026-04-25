use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{error::AppError, state::AppState};

pub async fn require_bearer_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "missing authorization header"))?;

    let provided_token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "invalid authorization scheme"))?;

    if provided_token != state.config.api_token {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid api token"));
    }

    Ok(next.run(request).await)
}

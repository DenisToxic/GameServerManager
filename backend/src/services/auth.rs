use std::sync::Arc;

use sqlx::PgPool;
use validator::Validate;

use crate::{
    config::AppConfig,
    models::{
        api::auth::{AuthResponse, LoginRequest, RegisterRequest},
        auth::{hash_password, issue_access_token, verify_password},
        db::UserRecord,
    },
};

use super::errors::AppError;

#[derive(Clone)]
pub struct AuthService {
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub async fn register(
        &self,
        pool: &PgPool,
        payload: RegisterRequest,
    ) -> Result<AuthResponse, AppError> {
        payload
            .validate()
            .map_err(|error| AppError::Validation(error.to_string()))?;

        let email = payload.email;
        let password_hash = hash_password(&payload.password).map_err(|error| {
            AppError::Internal(anyhow::anyhow!("failed to hash password: {error}"))
        })?;

        let existing: Option<uuid::Uuid> =
            sqlx::query_scalar(r#"SELECT id FROM users WHERE email = $1"#)
                .bind(email.clone())
                .fetch_optional(pool)
                .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("email already exists".to_owned()));
        }

        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (id, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, created_at
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        issue_access_token(&self.config, user.id, user.email).map_err(|error| {
            AppError::Internal(anyhow::anyhow!("failed to issue access token: {error}"))
        })
    }

    pub async fn login(
        &self,
        pool: &PgPool,
        payload: LoginRequest,
    ) -> Result<AuthResponse, AppError> {
        payload
            .validate()
            .map_err(|error| AppError::Validation(error.to_string()))?;

        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(payload.email)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

        if !verify_password(&payload.password, &user.password_hash) {
            return Err(AppError::Unauthorized);
        }

        issue_access_token(&self.config, user.id, user.email).map_err(|error| {
            AppError::Internal(anyhow::anyhow!("failed to issue access token: {error}"))
        })
    }
}

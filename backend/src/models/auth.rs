use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{
    config::AppConfig,
    middleware::auth::AuthenticatedUser,
    models::api::auth::{AuthResponse, AuthUserResponse},
};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn issue_access_token(
    config: &AppConfig,
    user_id: uuid::Uuid,
    email: String,
) -> Result<AuthResponse, jsonwebtoken::errors::Error> {
    let expires_at = Utc::now() + Duration::minutes(config.access_token_ttl_minutes);
    let claims = AuthenticatedUser {
        user_id,
        email: email.clone(),
        exp: expires_at.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(AuthResponse {
        access_token,
        token_type: "Bearer",
        expires_in_seconds: config.access_token_ttl_minutes * 60,
        user: AuthUserResponse { id: user_id, email },
    })
}

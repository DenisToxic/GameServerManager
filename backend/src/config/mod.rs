use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub access_token_ttl_minutes: i64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = ::config::Config::builder()
            .add_source(::config::Environment::default())
            .build()
            .context("failed to load configuration")?;

        Ok(Self {
            host: config
                .get_string("APP_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_owned()),
            port: config.get_int("APP_PORT").unwrap_or(8080) as u16,
            database_url: config
                .get_string("DATABASE_URL")
                .context("DATABASE_URL is required")?,
            jwt_secret: config
                .get_string("JWT_SECRET")
                .context("JWT_SECRET is required")?,
            access_token_ttl_minutes: config.get_int("ACCESS_TOKEN_TTL_MINUTES").unwrap_or(60),
        })
    }
}

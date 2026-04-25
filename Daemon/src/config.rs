use std::{env, path::PathBuf};

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub api_token: String,
    pub state_file: PathBuf,
    pub managed_label: String,
    pub minecraft_image: String,
    pub rust_image: String,
    pub hytale_image: String,
    pub base_volume_dir: PathBuf,
    pub default_working_dir: String,
    pub hytale_internal_port: u16,
    pub hytale_start_command: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let bind_addr = env::var("DAEMON_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let api_token = env::var("DAEMON_API_TOKEN")
            .map_err(|_| AppError::internal("DAEMON_API_TOKEN must be set"))?;
        let state_file = env::var("DAEMON_STATE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/game-host-daemon/state.json"));
        let default_image =
            env::var("DAEMON_DEFAULT_IMAGE").unwrap_or_else(|_| "ubuntu:22.04".to_string());
        let minecraft_image = env::var("DAEMON_MINECRAFT_IMAGE")
            .unwrap_or_else(|_| "didstopia/minecraft-server:latest".to_string());
        let rust_image =
            env::var("DAEMON_RUST_IMAGE").unwrap_or_else(|_| "ipajudd/rustgs:latest".to_string());
        let hytale_image =
            env::var("DAEMON_HYTALE_IMAGE").unwrap_or_else(|_| default_image.clone());
        let base_volume_dir = env::var("DAEMON_BASE_VOLUME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/var/lib/game-host-daemon/servers"));
        let default_working_dir = env::var("DAEMON_DEFAULT_WORKING_DIR")
            .unwrap_or_else(|_| "/home/container".to_string());
        let default_internal_port = env::var("DAEMON_DEFAULT_INTERNAL_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(25565);
        let default_start_command: Vec<String> = env::var("DAEMON_DEFAULT_START_COMMAND")
            .unwrap_or_else(|_| "sleep infinity".to_string())
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        let hytale_internal_port = env::var("DAEMON_HYTALE_INTERNAL_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default_internal_port);
        let hytale_start_command: Vec<String> = env::var("DAEMON_HYTALE_START_COMMAND")
            .unwrap_or_else(|_| default_start_command.join(" "))
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();

        Ok(Self {
            bind_addr,
            api_token,
            state_file,
            managed_label: "game-host-daemon.managed".to_string(),
            minecraft_image,
            rust_image,
            hytale_image,
            base_volume_dir,
            default_working_dir,
            hytale_internal_port,
            hytale_start_command,
        })
    }
}

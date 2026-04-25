use std::collections::HashMap;

use serde_json::Value;

use crate::{
    config::Config,
    error::AppError,
    models::{BackendCreateContainerRequest, ContainerPortMapping, ContainerSpec},
};

struct GamePreset {
    image: String,
    working_dir: String,
    ports: Vec<ContainerPortMapping>,
    environment: HashMap<String, String>,
    cmd: Vec<String>,
    entrypoint: Vec<String>,
}

fn memory_gib_string(memory_limit_mb: i32) -> String {
    let gib = ((memory_limit_mb.max(1024) + 1023) / 1024).max(1);
    format!("{gib}G")
}

fn settings_object(
    request: &BackendCreateContainerRequest,
) -> Option<&serde_json::Map<String, Value>> {
    request.server_settings.as_object()
}

fn setting_string(request: &BackendCreateContainerRequest, key: &str) -> Option<String> {
    settings_object(request)?
        .get(key)?
        .as_str()
        .map(ToOwned::to_owned)
}

fn setting_i64(request: &BackendCreateContainerRequest, key: &str) -> Option<i64> {
    settings_object(request)?.get(key)?.as_i64()
}

fn setting_bool(request: &BackendCreateContainerRequest, key: &str) -> Option<bool> {
    settings_object(request)?.get(key)?.as_bool()
}

pub fn build_container_spec(
    config: &Config,
    request: &BackendCreateContainerRequest,
) -> Result<ContainerSpec, AppError> {
    let preset = match request.game_kind.as_str() {
        "minecraft" => minecraft_preset(config, request),
        "rust" => rust_preset(config, request),
        "hytale" => hytale_preset(config, request),
        _ => {
            return Err(AppError::internal(format!(
                "unsupported game_kind: {}",
                request.game_kind
            )))
        }
    };

    let volume_host_path = config.base_volume_dir.join(request.server_id.to_string());

    Ok(ContainerSpec {
        server_id: request.server_id.to_string(),
        image: preset.image,
        environment: preset.environment,
        ports: preset.ports,
        memory_limit_mb: Some(request.memory_limit_mb as i64),
        cpu_limit: Some(request.cpu_limit_percent as f64 / 100.0),
        working_dir: preset.working_dir,
        volume_host_path: volume_host_path
            .into_os_string()
            .into_string()
            .map_err(|_| AppError::internal("volume path is not valid utf-8"))?,
        cmd: preset.cmd,
        entrypoint: preset.entrypoint,
        container_name: Some(format!("gsm-{}-{}", request.game_kind, request.server_id)),
    })
}

fn minecraft_preset(config: &Config, request: &BackendCreateContainerRequest) -> GamePreset {
    let memory = memory_gib_string(request.memory_limit_mb);
    let mut environment = HashMap::from([
        ("MINECRAFT_SERVER_MEMORY_MIN".to_string(), memory.clone()),
        ("MINECRAFT_SERVER_MEMORY_MAX".to_string(), memory),
        (
            "MINECRAFT_SERVER_AGREE_EULA".to_string(),
            "true".to_string(),
        ),
        (
            "MINECRAFT_SERVER_ARGUMENTS".to_string(),
            setting_string(request, "startup_arguments").unwrap_or_else(|| "nogui".to_string()),
        ),
    ]);

    if setting_bool(request, "rcon_enabled").unwrap_or(false) {
        environment.insert(
            "MINECRAFT_SERVER_RCON_ENABLE".to_string(),
            "true".to_string(),
        );
        environment.insert(
            "MINECRAFT_SERVER_RCON_PORT".to_string(),
            setting_i64(request, "rcon_port")
                .unwrap_or(25575)
                .to_string(),
        );
        if let Some(password) = setting_string(request, "rcon_password") {
            if !password.trim().is_empty() {
                environment.insert("MINECRAFT_SERVER_RCON_PASSWORD".to_string(), password);
            }
        }
    }

    if let Some(custom_jar) = setting_string(request, "custom_jar") {
        if !custom_jar.trim().is_empty() {
            environment.insert("MINECRAFT_SERVER_CUSTOM_JAR".to_string(), custom_jar);
        }
    }

    GamePreset {
        image: config.minecraft_image.clone(),
        working_dir: "/app".to_string(),
        ports: vec![ContainerPortMapping {
            container_port: 25565,
            host_port: request.allocated_port as u16,
            protocol: "tcp".to_string(),
            host_ip: "0.0.0.0".to_string(),
        }],
        environment,
        cmd: Vec::new(),
        entrypoint: Vec::new(),
    }
}

fn rust_preset(config: &Config, request: &BackendCreateContainerRequest) -> GamePreset {
    let max_players = setting_i64(request, "max_players").unwrap_or(50);
    let world_size = setting_i64(request, "world_size").unwrap_or(3500);
    let seed = setting_i64(request, "seed").unwrap_or(12345);
    let rcon_port = setting_i64(request, "rcon_port").unwrap_or(28016);
    let query_port = setting_i64(request, "query_port").unwrap_or(28016);
    let app_port = setting_i64(request, "app_port").unwrap_or(28082);
    let rcon_password = setting_string(request, "rcon_password")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "change-me-now".to_string());
    let mod_framework = setting_string(request, "mod_framework")
        .map(|value| value.to_ascii_lowercase())
        .or_else(|| {
            setting_bool(request, "oxide_enabled")
                .and_then(|enabled| enabled.then(|| "oxide".to_string()))
        })
        .unwrap_or_else(|| "vanilla".to_string());

    let mut environment = HashMap::from([
        ("SERVER_IDENTITY".to_string(), request.server_id.to_string()),
        (
            "SERVER_PORT".to_string(),
            request.allocated_port.to_string(),
        ),
        ("SERVER_QUERYPORT".to_string(), query_port.to_string()),
        ("RCON_PORT".to_string(), rcon_port.to_string()),
        ("RCON_PASSWORD".to_string(), rcon_password),
        ("RCON_WEB".to_string(), "1".to_string()),
        ("SERVER_HOSTNAME".to_string(), request.name.clone()),
        ("SERVER_MAXPLAYERS".to_string(), max_players.to_string()),
        ("SERVER_SEED".to_string(), seed.to_string()),
        ("SERVER_WORLDSIZE".to_string(), world_size.to_string()),
        ("APP_PORT".to_string(), app_port.to_string()),
        ("SERVER_SECURE".to_string(), "true".to_string()),
        ("SERVER_ENCRYPTION".to_string(), "1".to_string()),
        (
            "OXIDE_MOD".to_string(),
            if mod_framework == "oxide" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
        (
            "CARBON_MOD".to_string(),
            if mod_framework == "carbon" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
        (
            "USE_OXIDE".to_string(),
            if mod_framework == "oxide" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
        (
            "USE_CARBON".to_string(),
            if mod_framework == "carbon" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
        (
            "FORCE_OXIDE_INSTALLATION".to_string(),
            if mod_framework == "oxide" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
        (
            "FORCE_CARBON_INSTALLATION".to_string(),
            if mod_framework == "carbon" {
                "true"
            } else {
                "false"
            }
            .to_string(),
        ),
    ]);

    if let Some(description) = setting_string(request, "description") {
        if !description.trim().is_empty() {
            environment.insert("SERVER_DESCRIPTION".to_string(), description);
        }
    }

    if let Some(website_url) = setting_string(request, "website_url") {
        if !website_url.trim().is_empty() {
            environment.insert("SERVER_URL".to_string(), website_url);
        }
    }

    GamePreset {
        image: config.rust_image.clone(),
        working_dir: "/serverdata".to_string(),
        ports: vec![
            ContainerPortMapping {
                container_port: 28015,
                host_port: request.allocated_port as u16,
                protocol: "udp".to_string(),
                host_ip: "0.0.0.0".to_string(),
            },
            ContainerPortMapping {
                container_port: 28015,
                host_port: request.allocated_port as u16,
                protocol: "tcp".to_string(),
                host_ip: "0.0.0.0".to_string(),
            },
            ContainerPortMapping {
                container_port: query_port as u16,
                host_port: query_port as u16,
                protocol: "udp".to_string(),
                host_ip: "0.0.0.0".to_string(),
            },
            ContainerPortMapping {
                container_port: query_port as u16,
                host_port: query_port as u16,
                protocol: "tcp".to_string(),
                host_ip: "0.0.0.0".to_string(),
            },
            ContainerPortMapping {
                container_port: app_port as u16,
                host_port: app_port as u16,
                protocol: "tcp".to_string(),
                host_ip: "0.0.0.0".to_string(),
            },
        ],
        environment,
        cmd: Vec::new(),
        entrypoint: Vec::new(),
    }
}

fn hytale_preset(config: &Config, request: &BackendCreateContainerRequest) -> GamePreset {
    let startup_command = setting_string(request, "startup_command")
        .unwrap_or_else(|| config.hytale_start_command.join(" "));

    GamePreset {
        image: config.hytale_image.clone(),
        working_dir: config.default_working_dir.clone(),
        ports: vec![ContainerPortMapping {
            container_port: config.hytale_internal_port,
            host_port: request.allocated_port as u16,
            protocol: "tcp".to_string(),
            host_ip: "0.0.0.0".to_string(),
        }],
        environment: HashMap::from([
            ("SERVER_ID".to_string(), request.server_id.to_string()),
            ("SERVER_NAME".to_string(), request.name.clone()),
            ("GAME_KIND".to_string(), "hytale".to_string()),
            (
                "ALLOCATED_PORT".to_string(),
                request.allocated_port.to_string(),
            ),
            (
                "MAX_PLAYERS".to_string(),
                setting_i64(request, "max_players")
                    .unwrap_or(32)
                    .to_string(),
            ),
        ]),
        cmd: startup_command
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect(),
        entrypoint: Vec::new(),
    }
}

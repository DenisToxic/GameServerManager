use std::collections::{HashMap, HashSet};

use bollard::{
    container::{
        Config as ContainerConfig, CreateContainerOptions, InspectContainerOptions,
        ListContainersOptions, LogOutput, LogsOptions, RemoveContainerOptions,
        RestartContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions,
    },
    errors::Error as BollardError,
    image::CreateImageOptions,
    models::{ContainerInspectResponse, HostConfig, PortBinding},
    Docker,
};
use chrono::{DateTime, Utc};
use futures_util::stream::TryStreamExt;
use http::StatusCode;
use tracing::{debug, info, warn};

use crate::{
    config::Config,
    error::AppError,
    game_presets::build_container_spec,
    models::{
        BackendCreateContainerRequest, ContainerLogLine, ContainerPortMapping, ContainerSpec,
        ContainerStatusResponse, CreateContainerResult,
    },
};

#[derive(Clone)]
pub struct DockerClient {
    docker: Docker,
    managed_label: String,
}

impl DockerClient {
    pub fn new(managed_label: String) -> Result<Self, AppError> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|error| AppError::internal(format!("failed to connect to docker: {error}")))?;

        Ok(Self {
            docker,
            managed_label,
        })
    }

    pub async fn create_container(
        &self,
        spec: &ContainerSpec,
    ) -> Result<CreateContainerResult, AppError> {
        self.ensure_image_present(&spec.image).await?;

        let container_name = spec
            .container_name
            .clone()
            .unwrap_or_else(|| format!("srv-{}", sanitize_name(&spec.server_id)));
        let port_bindings = build_port_bindings(&spec.ports);
        let exposed_ports = build_exposed_ports(&spec.ports);
        let env = spec
            .environment
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>();
        let binds = vec![format!("{}:{}", spec.volume_host_path, spec.working_dir)];
        let labels = HashMap::from([
            (self.managed_label.clone(), "true".to_string()),
            (
                "game-host-daemon.server_id".to_string(),
                spec.server_id.clone(),
            ),
        ]);

        let host_config = HostConfig {
            memory: spec.memory_limit_mb.map(|value| value * 1024 * 1024),
            nano_cpus: spec
                .cpu_limit
                .map(|value| (value * 1_000_000_000_f64) as i64),
            port_bindings: (!port_bindings.is_empty()).then_some(port_bindings),
            binds: Some(binds),
            auto_remove: Some(false),
            ..Default::default()
        };

        let container_config = ContainerConfig {
            image: Some(spec.image.clone()),
            env: (!env.is_empty()).then_some(env),
            host_config: Some(host_config),
            working_dir: Some(spec.working_dir.clone()),
            exposed_ports: (!exposed_ports.is_empty()).then_some(exposed_ports),
            labels: Some(labels),
            cmd: (!spec.cmd.is_empty()).then_some(spec.cmd.clone()),
            entrypoint: (!spec.entrypoint.is_empty()).then_some(spec.entrypoint.clone()),
            tty: Some(false),
            attach_stdout: Some(false),
            attach_stderr: Some(false),
            ..Default::default()
        };

        let result = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name,
                    platform: None,
                }),
                container_config,
            )
            .await
            .map_err(map_docker_error)?;

        info!(server_id = %spec.server_id, container_id = %result.id, "container created");

        Ok(CreateContainerResult {
            server_id: spec.server_id.clone(),
            container_id: result.id,
            status: "stopped".to_string(),
        })
    }

    pub async fn create_backend_container(
        &self,
        config: &Config,
        request: &BackendCreateContainerRequest,
    ) -> Result<CreateContainerResult, AppError> {
        let container_spec = build_container_spec(config, request)?;
        self.create_container(&container_spec).await
    }

    async fn ensure_image_present(&self, image: &str) -> Result<(), AppError> {
        if self.docker.inspect_image(image).await.is_ok() {
            return Ok(());
        }

        info!(image, "docker image missing locally, pulling");
        let options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });
        let mut stream = self.docker.create_image(options, None, None);

        while stream.try_next().await.map_err(map_docker_error)?.is_some() {}

        Ok(())
    }

    pub async fn start_container(&self, container_id: &str) -> Result<(), AppError> {
        self.docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(map_docker_error)?;

        info!(container_id, "container started");
        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<(), AppError> {
        self.docker
            .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
            .await
            .map_err(map_docker_error)?;

        info!(container_id, "container stopped");
        Ok(())
    }

    pub async fn restart_container(&self, container_id: &str) -> Result<(), AppError> {
        self.docker
            .restart_container(container_id, Some(RestartContainerOptions { t: 10 }))
            .await
            .map_err(map_docker_error)?;

        info!(container_id, "container restarted");
        Ok(())
    }

    pub async fn delete_container(&self, container_id: &str) -> Result<(), AppError> {
        self.remove_container(container_id).await
    }

    pub async fn inspect_container(
        &self,
        container_id: &str,
    ) -> Result<ContainerInspectResponse, AppError> {
        self.docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await
            .map_err(map_docker_error)
    }

    pub async fn fetch_status(
        &self,
        server_id: &str,
        container_id: &str,
    ) -> Result<ContainerStatusResponse, AppError> {
        let inspect = self.inspect_container(container_id).await?;
        let state = inspect
            .state
            .as_ref()
            .ok_or_else(|| AppError::internal("docker inspect response missing state"))?;

        let mut stats_stream = self.docker.stats(
            container_id,
            Some(StatsOptions {
                stream: false,
                one_shot: true,
            }),
        );

        let stats = stats_stream.try_next().await.map_err(map_docker_error)?;
        let cpu_usage_percent = stats.as_ref().and_then(calculate_cpu_percentage);
        let memory_usage_bytes = stats.and_then(|value| value.memory_stats.usage);

        Ok(ContainerStatusResponse {
            server_id: server_id.to_string(),
            container_id: container_id.to_string(),
            state: if state.running.unwrap_or(false) {
                "running".to_string()
            } else {
                "stopped".to_string()
            },
            cpu_usage_percent,
            memory_usage_bytes,
        })
    }

    pub async fn fetch_logs(
        &self,
        container_id: &str,
        tail: usize,
    ) -> Result<Vec<ContainerLogLine>, AppError> {
        let mut logs_stream = self.docker.logs(
            container_id,
            Some(LogsOptions {
                follow: false,
                stdout: true,
                stderr: true,
                timestamps: true,
                tail: tail.to_string(),
                since: 0,
                until: 0,
            }),
        );
        let mut logs = Vec::new();

        while let Some(chunk) = logs_stream.try_next().await.map_err(map_docker_error)? {
            match chunk {
                LogOutput::StdOut { message }
                | LogOutput::StdErr { message }
                | LogOutput::Console { message }
                | LogOutput::StdIn { message } => {
                    logs.extend(parse_log_lines(&message));
                }
            }
        }

        Ok(logs)
    }

    pub async fn reconcile_managed_containers(
        &self,
        expected_container_ids: &HashSet<String>,
    ) -> Result<HashSet<String>, AppError> {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("{}=true", self.managed_label)],
        );

        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .map_err(map_docker_error)?;

        let mut discovered = HashSet::new();

        for container in containers {
            if let Some(container_id) = container.id {
                if expected_container_ids.contains(&container_id) {
                    debug!(%container_id, "managed container still tracked");
                    discovered.insert(container_id);
                    continue;
                }

                warn!(%container_id, "removing orphaned managed container");
                self.remove_container(&container_id).await?;
            }
        }

        Ok(discovered)
    }

    async fn remove_container(&self, container_id: &str) -> Result<(), AppError> {
        self.docker
            .remove_container(
                container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(map_docker_error)?;

        info!(container_id, "container removed");
        Ok(())
    }
}

fn sanitize_name(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => character,
            _ => '-',
        })
        .collect()
}

fn build_port_bindings(
    ports: &[ContainerPortMapping],
) -> HashMap<String, Option<Vec<PortBinding>>> {
    ports
        .iter()
        .map(|port| {
            (
                port.key(),
                Some(vec![PortBinding {
                    host_ip: Some(port.host_ip.clone()),
                    host_port: Some(port.host_port.to_string()),
                }]),
            )
        })
        .collect()
}

fn build_exposed_ports(ports: &[ContainerPortMapping]) -> HashMap<String, HashMap<(), ()>> {
    ports
        .iter()
        .map(|port| (port.key(), HashMap::new()))
        .collect()
}

fn calculate_cpu_percentage(stats: &bollard::container::Stats) -> Option<f64> {
    let cpu_delta = stats
        .cpu_stats
        .cpu_usage
        .total_usage
        .checked_sub(stats.precpu_stats.cpu_usage.total_usage)? as f64;
    let system_delta = stats
        .cpu_stats
        .system_cpu_usage?
        .checked_sub(stats.precpu_stats.system_cpu_usage?)? as f64;
    let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

    if system_delta <= 0.0 {
        return None;
    }

    Some((cpu_delta / system_delta) * online_cpus * 100.0)
}

fn parse_log_lines(bytes: &[u8]) -> Vec<ContainerLogLine> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|raw_line| {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                return None;
            }

            if let Some((timestamp, line)) = trimmed.split_once(' ') {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp) {
                    return Some(ContainerLogLine {
                        timestamp: timestamp.with_timezone(&Utc),
                        line: line.to_string(),
                    });
                }
            }

            Some(ContainerLogLine {
                timestamp: Utc::now(),
                line: trimmed.to_string(),
            })
        })
        .collect()
}

fn map_docker_error(error: BollardError) -> AppError {
    match error {
        BollardError::DockerResponseServerError {
            status_code,
            message,
        } => AppError::new(
            StatusCode::from_u16(status_code).unwrap_or(StatusCode::BAD_GATEWAY),
            message,
        ),
        other => AppError::internal(format!("docker error: {other}")),
    }
}

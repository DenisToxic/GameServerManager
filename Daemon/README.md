# Game Host Daemon

Rust daemon for managing Docker containers on a single Linux host. The backend talks only to this daemon over an authenticated HTTP API; the daemon is the only component that touches Docker.

## Features

- Bearer-token authentication on every endpoint
- Container lifecycle management via Docker
- Persisted `server_id -> container_id` mapping for restart recovery
- Startup reconciliation that removes orphaned managed containers
- Backend-compatible API contract for the GSM control plane
- Structured logging with `tracing`

## Environment

- `DAEMON_BIND_ADDR` - HTTP bind address, default `0.0.0.0:8080`
- `DAEMON_API_TOKEN` - required shared API token
- `DAEMON_STATE_FILE` - path for persisted state, default `/var/lib/game-host-daemon/state.json`
- `DAEMON_DEFAULT_IMAGE` - default image used for backend-issued create requests
- `DAEMON_MINECRAFT_IMAGE` - image used for Minecraft servers, default `itzg/minecraft-server:latest`
- `DAEMON_RUST_IMAGE` - image used for Rust servers, default `ipajudd/rustgs:latest`
- `DAEMON_HYTALE_IMAGE` - image used for Hytale servers, defaults to `DAEMON_DEFAULT_IMAGE`
- `DAEMON_BASE_VOLUME_DIR` - host directory under which per-server volumes are created
- `DAEMON_DEFAULT_WORKING_DIR` - container working directory mount target
- `DAEMON_DEFAULT_INTERNAL_PORT` - container port mapped to the allocated host port
- `DAEMON_DEFAULT_START_COMMAND` - whitespace-delimited default command for created containers
- `DAEMON_HYTALE_INTERNAL_PORT` - container port used by the Hytale preset, defaults to `DAEMON_DEFAULT_INTERNAL_PORT`
- `DAEMON_HYTALE_START_COMMAND` - command used by the Hytale preset, defaults to `DAEMON_DEFAULT_START_COMMAND`

## API

All requests require:

```http
Authorization: Bearer <token>
```

### `POST /containers/create`

Creates a managed container but does not start it.

```json
{
  "server_id": "00000000-0000-0000-0000-000000000001",
  "name": "alpha",
  "game_kind": "minecraft",
  "allocated_port": 25565,
  "memory_limit_mb": 2048,
  "cpu_limit_percent": 200
}
```

### `POST /containers/start`
### `POST /containers/stop`
### `POST /containers/restart`

```json
{
  "container_id": "1f2e3d4c5b6a"
}
```

### `GET /containers/:id/status`

`id` can be the tracked `server_id` or the managed `container_id`.

### `GET /containers/:id/logs?tail=200`

`id` can be the tracked `server_id` or the managed `container_id`.

## Run

```bash
cargo run --release
```

## Deployment

Use the included systemd unit template at `deploy/game-host-daemon.service` with an environment file such as `deploy/game-host-daemon.env.example`.

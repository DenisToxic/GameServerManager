# GSM Backend

Single-node game/server hosting panel backend built with Rust, Axum, PostgreSQL, SQLx, and JWT auth.

## What is included

- Layered Axum REST API
- SQL migrations for `users`, `nodes`, `servers`, and `server_logs`
- JWT access-token auth
- `reqwest` node daemon client for container lifecycle operations
- Validation and structured error handling

## Endpoints

- `POST /auth/register`
- `POST /auth/login`
- `GET /servers`
- `POST /servers`
- `GET /servers/:id`
- `POST /servers/:id/start`
- `POST /servers/:id/stop`
- `POST /servers/:id/restart`
- `DELETE /servers/:id`
- `GET /servers/:id/logs`
- `GET /servers/:id/status`

## Run

1. Copy `.env.example` to `.env`
2. Apply the SQL files in `migrations/` with SQLx or your migration runner
3. Start the API:

```bash
cargo run
```

## Deployment

A production systemd unit template is included at `deploy/gsm-backend.service` and a matching environment template at `deploy/.env.production.example`.

After applying the schema migrations, grant the runtime database role access to the application tables:

```bash
psql -d gsm -f deploy/grant-gsm-backend.sql
```

This is required when the tables are created by a different Postgres role than the one used in `DATABASE_URL`.

`POST /servers` now requires a `game_kind` of `minecraft`, `rust`, or `hytale` so the daemon can create a game-specific container preset.

## SQLx note

The project uses SQLx with typed queries and is structured so you can move to macro-checked SQL once a database or offline cache is available. If you want compile-time query validation, set `DATABASE_URL`, switch the queries back to SQLx macros, and run:

```bash
cargo sqlx prepare
```

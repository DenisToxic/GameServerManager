CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    api_url TEXT NOT NULL,
    api_token TEXT NOT NULL
);

CREATE TABLE servers (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    node_id INTEGER NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('stopped', 'running', 'starting', 'stopping', 'error')),
    docker_container_id TEXT,
    allocated_port INTEGER NOT NULL UNIQUE,
    memory_limit_mb INTEGER NOT NULL,
    cpu_limit_percent INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE server_logs (
    id BIGSERIAL PRIMARY KEY,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    line TEXT NOT NULL,
    UNIQUE (server_id, timestamp, line)
);

CREATE INDEX idx_servers_user_id ON servers(user_id);
CREATE INDEX idx_server_logs_server_id_timestamp ON server_logs(server_id, timestamp DESC);

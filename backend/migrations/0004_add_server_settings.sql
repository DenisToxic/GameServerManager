ALTER TABLE servers
ADD COLUMN server_settings JSONB NOT NULL DEFAULT '{}'::jsonb;

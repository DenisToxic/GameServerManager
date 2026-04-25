ALTER TABLE servers
ADD COLUMN game_kind TEXT NOT NULL DEFAULT 'minecraft';

ALTER TABLE servers
ADD CONSTRAINT servers_game_kind_check
CHECK (game_kind IN ('minecraft', 'rust', 'hytale'));

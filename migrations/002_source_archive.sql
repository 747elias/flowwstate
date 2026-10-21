CREATE TABLE IF NOT EXISTS chunk_versions(id TEXT PRIMARY KEY, data TEXT NOT NULL);
INSERT OR IGNORE INTO chunk_versions(id,data) SELECT id,data FROM chunks;
PRAGMA user_version=2;

CREATE TABLE IF NOT EXISTS workflow_entries(
    collection TEXT NOT NULL,
    id TEXT NOT NULL,
    data TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY(collection,id)
);
CREATE TABLE IF NOT EXISTS chat_agents(
    chat_id TEXT PRIMARY KEY REFERENCES chats(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL
);
PRAGMA user_version=3;

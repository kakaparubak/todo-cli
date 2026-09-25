use anyhow::Context;
use rusqlite::Connection;
use std::path::PathBuf;
use std::fs;

pub fn db_path() -> anyhow::Result<PathBuf> {
    let mut dir = dirs::data_dir().context("No data dir in system.")?;
    dir.push("todo-cli");
    fs::create_dir_all(&dir).context("Creating data dir.")?;
    dir.push("todos.db");
    Ok(dir)
}

pub fn open_and_migrate(path: &std::path::Path) -> anyhow::Result<Connection> {
    let conn = Connection::open(path).context("Failed opening connection.")?;
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            priority TEXT NOT NULL DEFAULT 'medium',
            due DATE,
            tags TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            completed_at TEXT,
            deleted_at TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_done ON todos (done);
        CREATE INDEX IF NOT EXISTS idx_priority ON todos (priority);
        CREATE INDEX IF NOT EXISTS idx_deleted_at ON todos (deleted_at);
        "#).context("Failed to run migration.")?;
    Ok(conn)
}
use anyhow::{Context};
use thiserror::Error;
use rusqlite::Connection;
use std::path::PathBuf;
use std::fs;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("todo #{0} not found")]
    NotFound(i64),

    #[error("todo #{0} is already done")]
    AlreadyDone(i64),

    #[error("todo #{0} is not deleted")]
    NotDeleted(i64),

    #[error("invalid priority: {0} (expected low|medium|high)")]
    InvalidPriority(String),

    #[error("invalid date: {0} (expected YYYY-MM-DD)")]
    InvalidDate(String),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Err(AppError::InvalidPriority(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

fn db_path() -> anyhow::Result<PathBuf> {
    let mut dir = dirs::data_dir().context("No data dir in system.")?;
    dir.push("todo-cli");
    fs::create_dir_all(&dir).context("Creating data dir.")?;
    dir.push("todos.db");
    Ok(dir)
}

fn open_and_migrate(path: &std::path::Path) -> anyhow::Result<Connection> {
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

fn main() -> anyhow::Result<()> {
    let dbpath = db_path()?;
    let db = open_and_migrate(&dbpath)?;
    println!("{:?}", db);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_inside_todo_cli_dir() -> anyhow::Result<()> {
        let p = db_path()?;
        assert!(p.ends_with("todos.db"));
        assert!(p.to_string_lossy().contains("todo-cli"));
        Ok(())
    }

    #[test]
    fn schema_creates_todos_table() -> anyhow::Result<()>{
    let conn = open_and_migrate(std::path::Path::new(":memory:"))?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='todos'",
            [],
            |r| r.get(0),
        )?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn priority_parse_known_values() {
        assert_eq!(Priority::parse("low").unwrap(), Priority::Low);
        assert_eq!(Priority::parse("high").unwrap(), Priority::High);
    }

    #[test]
    fn priority_parse_unknown_returns_error() {
        let err = Priority::parse("urgent").unwrap_err();
        assert!(matches!(err, AppError::InvalidPriority(s) if s == "urgent"));
    }
}
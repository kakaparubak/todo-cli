mod error;
mod db;
mod models;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let path = db::db_path()?;
    let conn = db::open_and_migrate(&path)?;
    match cli.command {
        Commands::Add(args) => println!("add: {:?}", args),
        Commands::List(args) => println!("list: {:?}", args),
        Commands::Done { id } => println!("done: {id}"),
        Commands::Undone { id } => println!("undone: {id}"),
        Commands::Edit(args) => println!("edit: {:?}", args),
        Commands::Delete { id } => println!("delete: {id}"),
        Commands::Restore { id } => println!("restore: {id}"),
    }
    let _ = conn;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use error::AppError;
    use models::Priority;

    #[test]
    fn db_path_inside_todo_cli_dir() -> anyhow::Result<()> {
        let p = db::db_path()?;
        assert!(p.ends_with("todos.db"));
        assert!(p.to_string_lossy().contains("todo-cli"));
        Ok(())
    }

    #[test]
    fn schema_creates_todos_table() -> anyhow::Result<()>{
    let conn = db::open_and_migrate(std::path::Path::new(":memory:"))?;
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
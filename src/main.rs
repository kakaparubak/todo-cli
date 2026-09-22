use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;

fn db_path() -> Result<PathBuf> {
    let mut dir = dirs::data_dir().context("No data dir in system.")?;
    dir.push("todo-cli");
    fs::create_dir_all(&dir).context("Creating data dir.")?;
    dir.push("todos.db");
    Ok(dir)
}

fn main() -> Result<()> {
    println!("{:?}", db_path()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_path_inside_todo_cli_dir() -> Result<()> {
        let p = db_path()?;
        assert!(p.ends_with("todos.db"));
        assert!(p.to_string_lossy().contains("todo-cli"));
        Ok(())
    }
}
use crate::cli::AddArgs;
use crate::error::{AppError, Result};
use crate::models::Priority;
use chrono::Utc;
use rusqlite::Connection;

fn tags_to_csv(tags: &[String]) -> String {
  
}
use thiserror::Error;

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

pub type Result<T> = std::result::Result<T, AppError>;
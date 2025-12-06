use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("not found")]
    NotFound,
    #[error("cache error: {0}")]
    CacheError(String),
    #[error("cassandra error: {0}")]
    Cassandra(String),
    #[error("redis error: {0}")]
    Redis(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

pub type AppResult<T> = Result<T, AppError>;

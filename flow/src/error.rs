use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for FlowError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        FlowError::Internal(err.to_string())
    }
}

impl From<&str> for FlowError {
    fn from(err: &str) -> Self {
        FlowError::Internal(err.to_string())
    }
}

impl From<String> for FlowError {
    fn from(err: String) -> Self {
        FlowError::Internal(err)
    }
}

pub type Result<T> = std::result::Result<T, FlowError>;


use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevUtilsError {
    #[error("API Request Failed: {0}")]
    ApiError(String),

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Agent Execution Failed: {0}")]
    AgentError(String),

    #[error("Vector Store Error: {0}")]
    VectorStoreError(String),

    #[error("Configuration Error: {0}")]
    ConfigError(String),

    #[error("General Error: {0}")]
    General(String),
}

// Convenient alias for modules using this error type
pub type Result<T> = std::result::Result<T, DevUtilsError>;

pub mod agent;
pub mod memory;
pub mod provider;
pub mod session;
pub mod tools;



pub type AiResult<T> = Result<T, AiError>;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Tool error: {0}")]
    Tool(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Session error: {0}")]
    Session(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Max tool rounds reached")]
    MaxToolRounds,
    #[error("No provider available")]
    NoProvider,
}

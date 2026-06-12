use serde::{Deserialize, Serialize};
use thiserror::Error;

impl From<std::io::Error> for LauncherError {
    fn from(e: std::io::Error) -> Self {
        LauncherError::Io(e.to_string())
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum LauncherError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Game not found: {0}")]
    NotFound(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Prefix error: {0}")]
    Prefix(String),
    #[error("Launch error: {0}")]
    Launch(String),
    #[error("Proton error: {0}")]
    Proton(String),
}

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Mutex lock failed: {0}")]
    MutexLockError(String),

    #[error("Downcast failed: {0}")]
    DowncastFailed(String),

    #[error("City not found: {0}")]
    CityNotFound(String),

    #[error("Environment variable error: {0}")]
    EnvVarError(String),
}

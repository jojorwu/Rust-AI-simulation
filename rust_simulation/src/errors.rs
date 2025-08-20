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

    #[error("Unwrap failed: {0}")]
    UnwrapFailed(String),

    #[error("Boxed error: {0}")]
    BoxedError(#[from] Box<dyn std::error::Error>),
}

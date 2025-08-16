use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SimulationError {
    SerializationError(String),
    TaskJoinError(String),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimulationError::SerializationError(e) => write!(f, "Serialization Error: {}", e),
            SimulationError::TaskJoinError(e) => write!(f, "Task Join Error: {}", e),
        }
    }
}

impl Error for SimulationError {}

impl From<serde_json::Error> for SimulationError {
    fn from(err: serde_json::Error) -> Self {
        SimulationError::SerializationError(err.to_string())
    }
}

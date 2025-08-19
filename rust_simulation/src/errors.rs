use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SimulationError {
    SerializationError(String),
    ComponentNotFound(String),
    IoError(String),
    Other(String),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimulationError::SerializationError(e) => write!(f, "Serialization Error: {}", e),
            SimulationError::ComponentNotFound(e) => write!(f, "Component Not Found: {}", e),
            SimulationError::IoError(e) => write!(f, "IO Error: {}", e),
            SimulationError::Other(e) => write!(f, "Other Error: {}", e),
        }
    }
}

impl Error for SimulationError {}

impl From<serde_json::Error> for SimulationError {
    fn from(err: serde_json::Error) -> Self {
        SimulationError::SerializationError(err.to_string())
    }
}

impl From<io::Error> for SimulationError {
    fn from(err: io::Error) -> Self {
        SimulationError::IoError(err.to_string())
    }
}

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RumblerError {
    #[error("configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("configuration error: {0}")]
    ConfigParse(String),

    #[error("unknown environment: {0}")]
    UnknownEnvironment(String),
    
}

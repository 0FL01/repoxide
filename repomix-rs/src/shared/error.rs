//! Error types

use thiserror::Error;

/// Repomix error types
#[derive(Error, Debug)]
pub enum RepomixError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Output error: {0}")]
    Output(String),
}

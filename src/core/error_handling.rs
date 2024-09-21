use thiserror::Error;

#[derive(Error, Debug)]
pub enum FerrousDBError {
    #[error("Table '{0}' not found")]
    TableNotFound(String),

    #[error("Column '{0}' not found")]
    ColumnNotFound(String),

    #[error("Type mismatch for column '{0}'")]
    TypeMismatch(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Table '{0}' already exists")]
    TableExists(String),

    #[error("Index on '{0}' not found")]
    IndexNotFound(String),

    #[error("Recover Error: '{0}'")]
    RecoveryError(String),
}

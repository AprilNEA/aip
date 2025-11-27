/// Error types for the filter parser
use thiserror::Error;

pub type Result<T> = std::result::Result<T, FilterError>;

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid field: {0}")]
    InvalidField(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

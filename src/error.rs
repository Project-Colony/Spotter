use thiserror::Error;

/// Typed error enum for Spotter operations.
///
/// Implements `Into<String>` so it can be used alongside the existing
/// `Result<T, String>` interfaces during gradual migration.
#[derive(Debug, Error)]
pub enum SpotterError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("{0}")]
    Other(String),
}

impl From<SpotterError> for String {
    fn from(e: SpotterError) -> Self {
        e.to_string()
    }
}

impl From<rusqlite::Error> for SpotterError {
    fn from(e: rusqlite::Error) -> Self {
        SpotterError::Database(e.to_string())
    }
}

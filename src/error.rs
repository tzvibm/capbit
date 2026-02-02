//! Error types for Capbit

/// The main error type for Capbit operations
#[derive(Debug, Clone)]
pub struct CapbitError(pub String);

impl std::fmt::Display for CapbitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CapbitError {}

/// Result type alias for Capbit operations
pub type Result<T> = std::result::Result<T, CapbitError>;

/// Convert any error to CapbitError
pub fn err<E: std::error::Error>(e: E) -> CapbitError {
    CapbitError(e.to_string())
}

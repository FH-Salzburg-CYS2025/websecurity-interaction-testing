//! Shared application error type.

use core::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// A simple owned error message used throughout the application.
#[derive(Debug)]
pub struct AppError(pub String);

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult { f.write_str(&self.0) }
}

// Error::provide requires an unstable feature gate; Error::type_id has a
// sealed parameter type that external implementors cannot name.
#[expect(
    clippy::missing_trait_methods,
    reason = "Error::provide and Error::type_id cannot be implemented by external code"
)]
impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}

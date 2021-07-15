/*!
One-line description.

More detailed description, with

# Example

*/

use std::fmt::{Display, Formatter};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum ErrorKind {
    LibraryOpenFailed(String, Box<dyn std::error::Error>),
    LibraryCloseFailed(String, Box<dyn std::error::Error>),
    SymbolNotFound(String, Box<dyn std::error::Error>),
    IncompatibleLibraryVersion(String),
    PluginRegistration(Box<dyn std::error::Error>),
    UnknownPluginManagerType(String),
}

#[derive(Debug)]
pub struct Error(ErrorKind);

pub type Result<T> = std::result::Result<T, Error>;

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ErrorKind::LibraryOpenFailed(path, error) =>
                    format!("Library '{}' failed to close; error: '{}'", path, error),
                ErrorKind::SymbolNotFound(name, in_library) => format!(
                    "Could not find symbol '{}' in library '{}'",
                    name, in_library
                ),
                ErrorKind::LibraryCloseFailed(path, error) =>
                    format!("Library '{}' failed to close; error: '{}'", path, error),
                ErrorKind::IncompatibleLibraryVersion(path) =>
                    format!("Library '{}' has incompatible version", path),
                ErrorKind::PluginRegistration(error) =>
                    format!("Plugin(s) failed to register; error: '{}'", error),
                ErrorKind::UnknownPluginManagerType(plugin_type) =>
                    format!("No Configured plugins for type '{}'", plugin_type),
            }
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ErrorKind> for Error {
    fn from(v: ErrorKind) -> Self {
        Self(v)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorKind::LibraryOpenFailed(_, error) => Some(error.as_ref()),
            ErrorKind::LibraryCloseFailed(_, error) => Some(error.as_ref()),
            ErrorKind::PluginRegistration(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

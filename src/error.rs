/*!
Provides the [`Error`](struct.Error.html), [`ErrorKind`](enum.ErrorKind.html), and
[`Result`](enum.Result.html) type used in the rest of this crate.
*/

use std::fmt::{Display, Formatter};

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Errors returned by functions and methods in this crate.
///
#[derive(Debug)]
pub enum ErrorKind {
    ///
    /// Failed to load the dynamic library specified by file name.
    /// The first parameter is the library path, the second is the underlying system error.
    ///
    LibraryOpenFailed(String, Box<dyn std::error::Error>),
    ///
    /// Failed to close the dynamic library and free any resources.
    /// The first parameter is the library path, the second is the underlying system error.
    ///
    LibraryCloseFailed(String, Box<dyn std::error::Error>),
    ///
    /// Failed to find the symbol within the dynamic library.
    /// The first parameter is the library path, the second is the underlying system error.
    ///
    SymbolNotFound(String, Box<dyn std::error::Error>),
    ///
    /// The plugin host and plugin library are incompatible.
    /// The parameter contains the path of the incompatible library.
    ///
    IncompatibleLibraryVersion(String),
    ///
    /// An error was reported by the plugin library when attempting to register a plugin.
    /// The parameter is the error the plugin library provided to the registrar.
    ///
    PluginRegistration(Box<dyn std::error::Error>),
    ///
    /// The plugin manager type is not known in the configuration.
    /// The parameter is the plugin type identifier that could not be found.
    ///
    UnknownPluginManagerType(String),
}

///
/// An implementation of `std::error::Error` using [`ErrorKind`](enum.ErrorKind.html).
///
#[derive(Debug)]
pub struct Error(ErrorKind);

///
/// `std::result::Result` constrained to always return the [`Error`](struct.Error.html)
/// type from this crate.
///
pub type Result<T> = std::result::Result<T, Error>;

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

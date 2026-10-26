//! Error and result types shared by the library and the CLI.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Every fallible operation of this crate returns this error type.
#[derive(Debug)]
pub enum Error {
    /// An I/O operation failed. `context` names the file or folder involved.
    Io {
        /// Description of the operation and the path it applied to.
        context: String,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// The `.osu` file could not be parsed by rosu-pp.
    Parse {
        /// Parser message.
        message: String,
    },
    /// An [`AnalyzeOptions`](crate::AnalyzeOptions) value is out of range.
    InvalidOptions(String),
    /// The analysis cache could not be read or written.
    Cache {
        /// Cache file involved.
        path: PathBuf,
        /// Reason the cache is unusable.
        message: String,
    },
    /// Writing or reading CSV failed.
    Csv(csv::Error),
    /// Serialising the output to JSON failed.
    Json(serde_json::Error),
}

impl Error {
    /// Creates an [`Error::Io`] with a human readable context.
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Parse { message } => write!(f, "failed to parse beatmap: {message}"),
            Self::InvalidOptions(message) => write!(f, "invalid options: {message}"),
            Self::Cache { path, message } => {
                write!(f, "cache {}: {message}", path.display())
            }
            Self::Csv(err) => write!(f, "CSV error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Csv(err) => Some(err),
            Self::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::io("I/O error", source)
    }
}

impl From<rosu_pp::ParseError> for Error {
    fn from(err: rosu_pp::ParseError) -> Self {
        Self::Parse {
            message: err.to_string(),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

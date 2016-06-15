//! Error and Result module.

use std::error as std_error;
use std::result as std_result;
use std::io;
use std::fmt::{self, Display, Formatter};

use serde_json;

// Bring the constructors of Error into scope so we can use them without an `Error::` incantation
use self::Error::{Io, Serde};

/// A Result alias often returned from methods that can fail for `fe_bucket` exclusive reasons.
pub type Result<T> = std_result::Result<T, Error>;

/// Errors that can occur during `fe_bucket` operations
#[derive(Debug)]
pub enum Error {
    /// Something went wrong internally while trying to perform IO.
    Io(io::Error),

    /// Problems with (de)serializing tables.
    ///
    /// `serde_json` makes no type-level distinction between serialization and deserialization
    /// errors, so we inherit that silliness.
    Serde(serde_json::Error)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Serde(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std_result::Result<(), fmt::Error> {
        match *self {
            Io(ref e) => {
                try!(write!(f, "Error performing IO: "));
                e.fmt(f)
            },
            Serde(ref e) => {
                try!(write!(f, "Error (de)serializing: "));
                e.fmt(f)
            }
        }
    }
}

impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Io(ref e) => e.description(),
            Serde(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&std_error::Error> {
        match *self {
            Io(ref e) => Some(e),
            Serde(ref e) => Some(e)
        }
    }
}

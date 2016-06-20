// Copyright 2016 The Fe_Bucket Project Developers. See the COPYRIGHT file at
// the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
// file may not be copied, modified, or distributed except according to those
// terms.

//! Error and Result module.

use std::error as std_error;
use std::result as std_result;
use std::io;
use std::fmt::{self, Display, Formatter};

use serde_json;

// Bring the constructors of Error into scope so we can use them without an `Error::` incantation
use self::Error::{Io, Serde, NoSuchTable};

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
    Serde(serde_json::Error),

    /// The user tried to read a table, but no such table exists.
    NoSuchTable(String)
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
            },
            NoSuchTable(ref t) =>
                write!(f, "Tried to open the table \"{}\", which does not exist.", t)
        }
    }
}

impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Io(ref e)      => e.description(),
            Serde(ref e)   => e.description(),
            NoSuchTable(_) => "Tried to open a table that doesn't exist"
        }
    }

    fn cause(&self) -> Option<&std_error::Error> {
        match *self {
            Io(ref e)      => Some(e),
            Serde(ref e)   => Some(e),
            NoSuchTable(_) => None
        }
    }
}

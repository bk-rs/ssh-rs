use core::fmt;
use std::io::Error as IoError;

use ssh2::Error as Ssh2Error;

//
#[derive(Debug)]
pub enum Error {
    Ssh2(Ssh2Error),
    Io(IoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

//
impl From<Ssh2Error> for Error {
    fn from(err: Ssh2Error) -> Self {
        Self::Ssh2(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use ssh2::Error as Ssh2Error;

//
#[derive(Debug)]
pub enum Error {
    Ssh2(Ssh2Error),
    Io(IoError),
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

//
impl Error {
    pub fn as_ssh2(&self) -> Option<&Ssh2Error> {
        match self {
            Self::Ssh2(err) => Some(err),
            _ => None,
        }
    }

    pub fn as_io(&self) -> Option<&IoError> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }

    pub fn as_other(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        match self {
            Self::Other(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

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

//
impl From<Error> for IoError {
    fn from(err: Error) -> Self {
        match err {
            Error::Ssh2(err) => IoError::new(IoErrorKind::Other, err),
            Error::Io(err) => err,
            Error::Other(err) => IoError::new(IoErrorKind::Other, err),
        }
    }
}

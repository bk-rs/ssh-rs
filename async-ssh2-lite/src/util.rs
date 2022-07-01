use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use ssh2::Error as Ssh2Error;

//
pub(crate) fn ssh2_error_is_would_block(err: &Ssh2Error) -> bool {
    IoError::from(Ssh2Error::from_errno(err.code())).kind() == IoErrorKind::WouldBlock
}

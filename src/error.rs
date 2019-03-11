use crate::message::error::Error as MessageError;
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct Error(ErrorKind);

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.kind().source()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    Message(MessageError),
    #[doc(hidden)]
    __NonExhaustive,
}

impl ErrorKind {}

impl StdError for ErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ErrorKind::Message(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Message(_) => write!(f, "message error"),
            ErrorKind::__NonExhaustive => unreachable!("invalid error"),
        }
    }
}

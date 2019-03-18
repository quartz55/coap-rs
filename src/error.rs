pub use crate::message::error::Error as MessageError;
use std::error::Error as StdError;
use std::fmt::{self, Write};
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

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

#[derive(Debug)]
pub enum ErrorKind {
    Message(MessageError),
    ServerIo(std::io::Error),
    #[doc(hidden)]
    __NonExhaustive,
}

impl ErrorKind {}

impl StdError for ErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ErrorKind::Message(ref err) => Some(err),
            ErrorKind::ServerIo(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Message(_) => write!(f, "message error"),
            ErrorKind::ServerIo(_) => write!(f, "server io error"),
            ErrorKind::__NonExhaustive => unreachable!("invalid error"),
        }
    }
}

impl<E> From<E> for Error
where
    E: Into<ErrorKind>,
{
    fn from(e: E) -> Self {
        e.into().into()
    }
}

impl From<MessageError> for ErrorKind {
    fn from(err: MessageError) -> Self {
        ErrorKind::Message(err)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> Self {
        ErrorKind::ServerIo(err)
    }
}

pub fn pprint_error(err: &dyn StdError) -> String {
    let mut s = String::new();
    let mut err = err;
    write!(&mut s, "{}", err).unwrap();
    while let Some(source) = err.source() {
        write!(&mut s, ": {}", source).unwrap();
        err = source;
    }
    s
}

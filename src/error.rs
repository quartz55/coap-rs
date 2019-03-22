pub use crate::message::error::MessageError;
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

    pub fn message(err: MessageError) -> Self {
        err.into()
    }

    pub fn server_io(err: std::io::Error) -> Self {
        err.into()
    }

    pub fn addr_unavailable() -> Self {
        Self(ErrorKind::AddrUnavailable)
    }

    pub fn response_timeout() -> Self {
        Self(ErrorKind::ResponseTimeout)
    }

    pub fn handler() -> Self {
        Self(ErrorKind::Handler)
    }

    pub fn request_cancelled() -> Self {
        Self(ErrorKind::RequestCancelled)
    }

    pub fn broken_channel<S>(reason: S) -> Self
    where
        S: Into<String>,
    {
        Self(ErrorKind::BrokenChannel(reason.into()))
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
    BrokenChannel(String),
    AddrUnavailable,
    ResponseTimeout,
    Handler,
    RequestCancelled,
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
            ErrorKind::BrokenChannel(ref reason) => write!(f, "broken channel: {}", reason),
            ErrorKind::AddrUnavailable => write!(f, "address unavailable"),
            ErrorKind::ResponseTimeout => write!(f, "response timeout"),
            ErrorKind::Handler => write!(f, "response handler failed"),
            ErrorKind::RequestCancelled => write!(f, "request cancelled by client"),
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

use super::header::Header;
use crate::params::VERSION;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Deref;
use std::ops::Range;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, MessageError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatError {
    UnknownVersion(u8),
    InvalidTokenLength(usize),
    InvalidEmptyCode(String),
    TokenLengthMismatch { actual: usize, expected: usize },
    InvalidOptionDelta,
    InvalidOptionLength,
    OptionLengthMismatch { actual: usize, expected: usize },
    InvalidOptionValue { range: Range<usize>, actual: usize },
    InvalidOption(String),
    UnexpectedPayloadMarker,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FormatError::*;
        match self {
            UnknownVersion(ver) => write!(f, "unkown version number {} (must be {})", ver, VERSION),
            InvalidTokenLength(tkl) => write!(
                f,
                "invalid token length {} (lengths 9 to 15 are reserved)",
                tkl
            ),
            InvalidEmptyCode(ref reason) => write!(f, "invalid empty code: {}", reason),
            TokenLengthMismatch { actual, expected } => write!(
                f,
                "invalid token (expected {} bytes, got {})",
                expected, actual
            ),
            InvalidOptionDelta => write!(f, "option delta 15 is reserved for the payload marker",),
            InvalidOptionLength => write!(f, "option length 15 is reserved for future use",),
            OptionLengthMismatch { actual, expected } => write!(
                f,
                "option value size mismatch (expected {} bytes, got {})",
                expected, actual
            ),
            InvalidOptionValue { range, actual } => write!(
                f,
                "invalid option value (expected {} - {} bytes, got {})",
                range.start, range.end, actual
            ),
            InvalidOption(ref reason) => write!(f, "invalid option: {}", reason),
            UnexpectedPayloadMarker => write!(f, "unexpected payload marker when payload is empty"),
        }
    }
}

impl StdError for FormatError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    MessageFormat(FormatError),
    PacketTooSmall(usize),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::MessageFormat(_) => write!(f, "invalid message format"),
            ErrorKind::PacketTooSmall(s) => {
                write!(f, "invalid CoAP packet size {} (minimum 4 bytes)", s)
            }
        }
    }
}

impl StdError for ErrorKind {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ErrorKind::MessageFormat(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<FormatError> for ErrorKind {
    fn from(e: FormatError) -> Self {
        ErrorKind::MessageFormat(e)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageError {
    kind: ErrorKind,
    header: Option<Header>,
}

impl MessageError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, header: None }
    }

    pub fn with_header(kind: ErrorKind, header: Header) -> Self {
        Self {
            kind,
            header: Some(header),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn header(&self) -> Option<&Header> {
        self.header.as_ref()
    }

    pub fn set_header(mut self, header: Header) -> Self {
        self.header = Some(header);
        self
    }

    pub fn take_header(&mut self) -> Option<Header> {
        self.header.take()
    }
}

impl Deref for MessageError {
    type Target = ErrorKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl StdError for MessageError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.kind.source()
    }
}

impl<K> From<K> for MessageError
where
    K: Into<ErrorKind>,
{
    fn from(kind: K) -> Self {
        Self::new(kind.into())
    }
}

impl<K> From<(K, Header)> for MessageError
where
    K: Into<ErrorKind>,
{
    fn from((kind, header): (K, Header)) -> Self {
        Self::with_header(kind.into(), header)
    }
}

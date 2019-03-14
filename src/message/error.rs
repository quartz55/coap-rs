use crate::params::VERSION;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Range;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

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
pub enum Error {
    MessageFormat(FormatError),
    PacketTooSmall(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MessageFormat(_) => write!(f, "invalid message format"),
            Error::PacketTooSmall(s) => {
                write!(f, "invalid CoAP packet size {} (minimum 4 bytes)", s)
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::MessageFormat(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error {
        Error::MessageFormat(e)
    }
}

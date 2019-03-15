use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidMethod(u8),
    InvalidResponseCode(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            InvalidMethod(class) => write!(f, "invalid method class (expected 0, got {})", class),
            InvalidResponseCode(class) => write!(
                f,
                "invalid response code class (expected 2, 4 or 5, got {})",
                class
            ),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RawCode(pub u8, pub u8);
impl RawCode {
    #[inline(always)]
    pub fn class(&self) -> u8 {
        self.0
    }
    #[inline(always)]
    pub fn detail(&self) -> u8 {
        self.1
    }
}
impl fmt::Debug for RawCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.0, self.1)
    }
}
impl fmt::Display for RawCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02}", self.0, self.1)
    }
}

impl RawCode {
    pub fn from_u8(raw: u8) -> Self {
        Self(code_class(raw), code_detail(raw))
    }
    #[inline(always)]
    pub fn as_u8(&self) -> u8 {
        (self.0 << 5) | self.1
    }
}

impl From<u8> for RawCode {
    fn from(raw: u8) -> Self {
        Self::from_u8(raw)
    }
}

impl From<(u8, u8)> for RawCode {
    fn from((class, detail): (u8, u8)) -> Self {
        Self(class, detail)
    }
}

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Unknown(u8),
}
impl Method {
    pub fn as_raw_code(&self) -> RawCode {
        match self {
            Method::Get => RawCode(0, 01),
            Method::Post => RawCode(0, 02),
            Method::Put => RawCode(0, 03),
            Method::Delete => RawCode(0, 04),
            Method::Unknown(dd) => RawCode(0, *dd),
        }
    }

    pub fn from_raw_code(raw_code: RawCode) -> Result<Self, Error> {
        match (raw_code.0, raw_code.1) {
            (0, 01) => Ok(Method::Get),
            (0, 02) => Ok(Method::Post),
            (0, 03) => Ok(Method::Put),
            (0, 04) => Ok(Method::Delete),
            (0, dd) if dd > 00 => Ok(Method::Unknown(dd)),
            (c, _) => Err(Error::InvalidMethod(c)),
        }
    }
}
impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Unknown(_) => write!(f, "{} UNKNOWN", self.as_raw_code()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SuccessCode {
    Created,
    Deleted,
    Valid,
    Changed,
    Content,
    Unknown(u8),
}

impl Default for SuccessCode {
    fn default() -> Self {
        SuccessCode::Content
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClientErrorCode {
    BadRequest,
    Unauthorized,
    BadOption,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    PreconditionFailed,
    RequestEntityTooLarge,
    UnsupportedContentFormat,
    Unknown(u8),
}

impl Default for ClientErrorCode {
    fn default() -> Self {
        ClientErrorCode::BadRequest
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServerErrorCode {
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    ProxyingNotSupported,
    Unknown(u8),
}

impl Default for ServerErrorCode {
    fn default() -> Self {
        ServerErrorCode::InternalServerError
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseCode {
    Success(SuccessCode),
    ClientError(ClientErrorCode),
    ServerError(ServerErrorCode),
}

impl ResponseCode {
    pub fn as_raw_code(&self) -> RawCode {
        use ClientErrorCode::*;
        use ServerErrorCode::*;
        use SuccessCode::*;
        match self {
            ResponseCode::Success(Created) => RawCode(2, 01),
            ResponseCode::Success(Deleted) => RawCode(2, 02),
            ResponseCode::Success(Valid) => RawCode(2, 03),
            ResponseCode::Success(Changed) => RawCode(2, 04),
            ResponseCode::Success(Content) => RawCode(2, 05),
            ResponseCode::Success(SuccessCode::Unknown(dd)) => RawCode(2, *dd),
            ResponseCode::ClientError(BadRequest) => RawCode(4, 00),
            ResponseCode::ClientError(Unauthorized) => RawCode(4, 01),
            ResponseCode::ClientError(BadOption) => RawCode(4, 02),
            ResponseCode::ClientError(Forbidden) => RawCode(4, 03),
            ResponseCode::ClientError(NotFound) => RawCode(4, 04),
            ResponseCode::ClientError(MethodNotAllowed) => RawCode(4, 05),
            ResponseCode::ClientError(NotAcceptable) => RawCode(4, 06),
            ResponseCode::ClientError(PreconditionFailed) => RawCode(4, 12),
            ResponseCode::ClientError(RequestEntityTooLarge) => RawCode(4, 13),
            ResponseCode::ClientError(UnsupportedContentFormat) => RawCode(4, 15),
            ResponseCode::ClientError(ClientErrorCode::Unknown(dd)) => RawCode(4, *dd),
            ResponseCode::ServerError(InternalServerError) => RawCode(5, 00),
            ResponseCode::ServerError(NotImplemented) => RawCode(5, 01),
            ResponseCode::ServerError(BadGateway) => RawCode(5, 02),
            ResponseCode::ServerError(ServiceUnavailable) => RawCode(5, 03),
            ResponseCode::ServerError(GatewayTimeout) => RawCode(5, 04),
            ResponseCode::ServerError(ProxyingNotSupported) => RawCode(5, 05),
            ResponseCode::ServerError(ServerErrorCode::Unknown(dd)) => RawCode(5, *dd),
        }
    }

    pub fn from_raw_code(raw_code: RawCode) -> Result<Self, Error> {
        use ClientErrorCode::*;
        use ServerErrorCode::*;
        use SuccessCode::*;
        match (raw_code.0, raw_code.1) {
            (2, 01) => Ok(ResponseCode::Success(Created)),
            (2, 02) => Ok(ResponseCode::Success(Deleted)),
            (2, 03) => Ok(ResponseCode::Success(Valid)),
            (2, 04) => Ok(ResponseCode::Success(Changed)),
            (2, 05) => Ok(ResponseCode::Success(Content)),
            (2, dd) => Ok(ResponseCode::Success(SuccessCode::Unknown(dd))),
            (4, 00) => Ok(ResponseCode::ClientError(BadRequest)),
            (4, 01) => Ok(ResponseCode::ClientError(Unauthorized)),
            (4, 02) => Ok(ResponseCode::ClientError(BadOption)),
            (4, 03) => Ok(ResponseCode::ClientError(Forbidden)),
            (4, 04) => Ok(ResponseCode::ClientError(NotFound)),
            (4, 05) => Ok(ResponseCode::ClientError(MethodNotAllowed)),
            (4, 06) => Ok(ResponseCode::ClientError(NotAcceptable)),
            (4, 12) => Ok(ResponseCode::ClientError(PreconditionFailed)),
            (4, 13) => Ok(ResponseCode::ClientError(RequestEntityTooLarge)),
            (4, 15) => Ok(ResponseCode::ClientError(UnsupportedContentFormat)),
            (4, dd) => Ok(ResponseCode::ClientError(ClientErrorCode::Unknown(dd))),
            (5, 00) => Ok(ResponseCode::ServerError(InternalServerError)),
            (5, 01) => Ok(ResponseCode::ServerError(NotImplemented)),
            (5, 02) => Ok(ResponseCode::ServerError(BadGateway)),
            (5, 03) => Ok(ResponseCode::ServerError(ServiceUnavailable)),
            (5, 04) => Ok(ResponseCode::ServerError(GatewayTimeout)),
            (5, 05) => Ok(ResponseCode::ServerError(ProxyingNotSupported)),
            (5, dd) => Ok(ResponseCode::ServerError(ServerErrorCode::Unknown(dd))),
            (c, _dd) => Err(Error::InvalidResponseCode(c)),
        }
    }

    pub fn name(&self) -> &'static str {
        use ClientErrorCode::*;
        use ServerErrorCode::*;
        use SuccessCode::*;
        match self {
            ResponseCode::Success(Created) => "Created",
            ResponseCode::Success(Deleted) => "Deleted",
            ResponseCode::Success(Valid) => "Valid",
            ResponseCode::Success(Changed) => "Changed",
            ResponseCode::Success(Content) => "Content",
            ResponseCode::ClientError(BadRequest) => "Bad Request",
            ResponseCode::ClientError(Unauthorized) => "Unauthorized",
            ResponseCode::ClientError(BadOption) => "Bad Option",
            ResponseCode::ClientError(Forbidden) => "Forbidden",
            ResponseCode::ClientError(NotFound) => "Not Found",
            ResponseCode::ClientError(MethodNotAllowed) => "Method Not Allowed",
            ResponseCode::ClientError(NotAcceptable) => "Not Acceptable",
            ResponseCode::ClientError(PreconditionFailed) => "Precondition Failed",
            ResponseCode::ClientError(RequestEntityTooLarge) => "Request Entity Too Large",
            ResponseCode::ClientError(UnsupportedContentFormat) => "Unsupported Content Format",
            ResponseCode::ServerError(InternalServerError) => "Internal Server Error",
            ResponseCode::ServerError(NotImplemented) => "Not Implemented",
            ResponseCode::ServerError(BadGateway) => "Bad Gateway",
            ResponseCode::ServerError(ServiceUnavailable) => "Service Unavailable",
            ResponseCode::ServerError(GatewayTimeout) => "Gateway Timeout",
            ResponseCode::ServerError(ProxyingNotSupported) => "Proxying Not Supported",
            ResponseCode::Success(SuccessCode::Unknown(_))
            | ResponseCode::ClientError(ClientErrorCode::Unknown(_))
            | ResponseCode::ServerError(ServerErrorCode::Unknown(_)) => "Unknown",
        }
    }
}

impl fmt::Display for ResponseCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.as_raw_code(), self.name())
    }
}

impl Default for ResponseCode {
    fn default() -> Self {
        SuccessCode::default().into()
    }
}

impl From<SuccessCode> for ResponseCode {
    fn from(code: SuccessCode) -> ResponseCode {
        ResponseCode::Success(code)
    }
}

impl From<ClientErrorCode> for ResponseCode {
    fn from(code: ClientErrorCode) -> ResponseCode {
        ResponseCode::ClientError(code)
    }
}

impl From<ServerErrorCode> for ResponseCode {
    fn from(code: ServerErrorCode) -> ResponseCode {
        ResponseCode::ServerError(code)
    }
}

#[inline(always)]
fn code_class(raw: u8) -> u8 {
    raw >> 5
}

#[inline(always)]
fn code_detail(raw: u8) -> u8 {
    raw & 0x1F
}

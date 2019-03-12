pub struct RawCode(pub u8, pub u8);

impl RawCode {
    #[inline(always)]
    pub fn class(raw: u8) -> u8 {
        raw >> 5
    }
    #[inline(always)]
    pub fn detail(raw: u8) -> u8 {
        raw & 0x1F
    }
}

impl RawCode {
    pub fn from_u8(raw: u8) -> Self {
        Self(Self::class(raw), Self::detail(raw))
    }
    #[inline(always)]
    pub fn as_u8(&self) -> u8 {
        (self.0 << 5) | self.1
    }
    #[inline(always)]
    pub fn is_request(&self) -> bool {
        self.0 == 0
    }
    #[inline(always)]
    pub fn is_response(&self) -> bool {
        self.0 == 2 || self.0 == 4 || self.0 == 5
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Code {
    Empty,
    Get,
    Post,
    Put,
    Delete,
    Created,
    Deleted,
    Valid,
    Changed,
    Content,
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
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    ProxyingNotSupported,
    Unknown((u8, u8)),
}

impl Code {
    pub fn from_u8(raw: u8) -> Self {
        match Self::cdd(raw) {
            (0, 00) => Code::Empty,
            (0, 01) => Code::Get,
            (0, 02) => Code::Post,
            (0, 03) => Code::Put,
            (0, 04) => Code::Delete,
            (2, 01) => Code::Created,
            (2, 02) => Code::Deleted,
            (2, 03) => Code::Valid,
            (2, 04) => Code::Changed,
            (2, 05) => Code::Content,
            (4, 00) => Code::BadRequest,
            (4, 01) => Code::Unauthorized,
            (4, 02) => Code::BadOption,
            (4, 03) => Code::Forbidden,
            (4, 04) => Code::NotFound,
            (4, 05) => Code::MethodNotAllowed,
            (4, 06) => Code::NotAcceptable,
            (4, 12) => Code::PreconditionFailed,
            (4, 13) => Code::RequestEntityTooLarge,
            (4, 15) => Code::UnsupportedContentFormat,
            (5, 00) => Code::InternalServerError,
            (5, 01) => Code::NotImplemented,
            (5, 02) => Code::BadGateway,
            (5, 03) => Code::ServiceUnavailable,
            (5, 04) => Code::GatewayTimeout,
            (5, 05) => Code::ProxyingNotSupported,
            cdd => Code::Unknown(cdd),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Code::Empty => Self::make(0, 00),
            Code::Get => Self::make(0, 01),
            Code::Post => Self::make(0, 02),
            Code::Put => Self::make(0, 03),
            Code::Delete => Self::make(0, 04),
            Code::Created => Self::make(2, 01),
            Code::Deleted => Self::make(2, 02),
            Code::Valid => Self::make(2, 03),
            Code::Changed => Self::make(2, 04),
            Code::Content => Self::make(2, 05),
            Code::BadRequest => Self::make(4, 00),
            Code::Unauthorized => Self::make(4, 01),
            Code::BadOption => Self::make(4, 02),
            Code::Forbidden => Self::make(4, 03),
            Code::NotFound => Self::make(4, 04),
            Code::MethodNotAllowed => Self::make(4, 05),
            Code::NotAcceptable => Self::make(4, 06),
            Code::PreconditionFailed => Self::make(4, 12),
            Code::RequestEntityTooLarge => Self::make(4, 13),
            Code::UnsupportedContentFormat => Self::make(4, 15),
            Code::InternalServerError => Self::make(5, 00),
            Code::NotImplemented => Self::make(5, 01),
            Code::BadGateway => Self::make(5, 02),
            Code::ServiceUnavailable => Self::make(5, 03),
            Code::GatewayTimeout => Self::make(5, 04),
            Code::ProxyingNotSupported => Self::make(5, 05),
            Code::Unknown((c, dd)) => Self::make(*c, *dd),
        }
    }

    #[inline(always)]
    pub fn make(class: u8, detail: u8) -> u8 {
        (class << 5) | detail
    }

    #[inline(always)]
    pub fn cdd(raw: u8) -> (u8, u8) {
        (Self::class(raw), Self::detail(raw))
    }

    #[inline(always)]
    pub fn class(raw: u8) -> u8 {
        raw >> 5
    }

    #[inline(always)]
    pub fn detail(raw: u8) -> u8 {
        raw & 0x1F
    }
}

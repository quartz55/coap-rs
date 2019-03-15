use super::body::Body;
use super::code::{Method, RawCode, ResponseCode};
use super::header::{Header, MessageType};
use super::message::{Message, MessageKind};
use super::option::Opts;
use super::token::Token;
use std::marker::PhantomData as PD;

pub struct Yes;
pub struct No;

pub trait IsSet {}
impl IsSet for Yes {}
impl IsSet for No {}

pub struct Empty;
pub struct Request;
pub struct Response;

pub trait Kind {}
impl Kind for Empty {}
impl Kind for Request {}
impl Kind for Response {}

pub trait NotEmpty {}
impl NotEmpty for Request {}
impl NotEmpty for Response {}

pub trait Confirmable {}
impl Confirmable for Request {}
impl Confirmable for Response {}

pub trait NonConfirmable {}
impl NonConfirmable for Request {}
impl NonConfirmable for Response {}

pub trait Acknowledgeable {}
impl Acknowledgeable for Response {}
impl Acknowledgeable for Empty {}

#[derive(Debug, Clone)]
pub struct MessageBuilder<KIND = Empty, MID = No, MTYPE = No>
where
    KIND: Kind,
    MID: IsSet,
    MTYPE: IsSet,
{
    _mid_set: PD<MID>,
    _mtype_set: PD<MTYPE>,
    _kind: PD<KIND>,

    mtype: MessageType,
    code: RawCode,
    mid: u16,
    token: Option<Token>,
    options: Option<Opts>,
    payload: Option<Vec<u8>>,
}

impl<KIND, MID, MTYPE> Default for MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind,
    MID: IsSet,
    MTYPE: IsSet,
{
    fn default() -> Self {
        MessageBuilder {
            _mid_set: PD {},
            _mtype_set: PD {},
            _kind: PD {},

            mtype: MessageType::Reset,
            code: RawCode(0, 00),
            mid: 0,
            token: None,
            options: None,
            payload: None,
        }
    }
}

impl MessageBuilder {
    pub fn empty() -> MessageBuilder<Empty, No, No> {
        Default::default()
    }

    pub fn ping(message_id: u16) -> MessageBuilder<Empty, Yes, Yes> {
        MessageBuilder::empty()
            .with_type(MessageType::Confirmable)
            .message_id(message_id)
    }

    pub fn reset(message_id: u16) -> MessageBuilder<Empty, Yes, Yes> {
        MessageBuilder::empty()
            .with_type(MessageType::Reset)
            .message_id(message_id)
    }

    pub fn request() -> MessageBuilder<Request, No, No> {
        MessageBuilder {
            code: Method::default().as_raw_code(),
            ..Default::default()
        }
    }

    pub fn response() -> MessageBuilder<Response, No, No> {
        MessageBuilder {
            code: ResponseCode::default().as_raw_code(),
            ..Default::default()
        }
    }
}

impl<KIND, MID, MTYPE> MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind,
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn message_id(self, message_id: u16) -> MessageBuilder<KIND, Yes, MTYPE> {
        MessageBuilder {
            _mid_set: PD {},
            _mtype_set: PD {},
            _kind: PD {},

            mtype: self.mtype,
            code: self.code,
            mid: message_id,
            token: self.token,
            options: self.options,
            payload: self.payload,
        }
    }

    fn with_type(self, mtype: MessageType) -> MessageBuilder<KIND, MID, Yes> {
        MessageBuilder {
            _mid_set: PD {},
            _mtype_set: PD {},
            _kind: PD {},

            mtype,
            code: self.code,
            mid: self.mid,
            token: self.token,
            options: self.options,
            payload: self.payload,
        }
    }
}

impl<KIND, MID, MTYPE> MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind + Confirmable,
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn confirmable(self) -> MessageBuilder<KIND, MID, Yes> {
        self.with_type(MessageType::Confirmable)
    }
}

impl<KIND, MID, MTYPE> MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind + NonConfirmable,
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn non_confirmable(self) -> MessageBuilder<KIND, MID, Yes> {
        self.with_type(MessageType::NonConfirmable)
    }
}

impl<KIND, MID, MTYPE> MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind + Acknowledgeable,
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn acknowledgement(self) -> MessageBuilder<KIND, MID, Yes> {
        self.with_type(MessageType::Acknowledgement)
    }
}

impl<KIND, MID, MTYPE> MessageBuilder<KIND, MID, MTYPE>
where
    KIND: Kind + NotEmpty,
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = Some(payload);
        self
    }
}

impl<MID, MTYPE> MessageBuilder<Request, MID, MTYPE>
where
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn method(mut self, method: Method) -> Self {
        self.code = method.as_raw_code();
        self
    }
}

impl<MID, MTYPE> MessageBuilder<Response, MID, MTYPE>
where
    MID: IsSet,
    MTYPE: IsSet,
{
    pub fn response_code(mut self, res_code: ResponseCode) -> Self {
        self.code = res_code.as_raw_code();
        self
    }
}

impl MessageBuilder<Empty, Yes, Yes> {
    pub fn build(self) -> Message {
        Message::empty(self.mtype, self.mid)
    }
}

impl<KIND> MessageBuilder<KIND, Yes, Yes>
where
    KIND: Kind + NotEmpty,
{
    fn make_header_body(self) -> (Header, Body) {
        let token = self.token.or_else(|| Some(Token::empty())).unwrap();
        let opts = self.options.or_else(|| Some(Opts::new())).unwrap();
        let header = Header::new(self.mtype, token.len(), self.code, self.mid);
        let body = Body::new(token, opts, self.payload);
        (header, body)
    }
}

impl MessageBuilder<Request, Yes, Yes> {
    pub fn build(self) -> Message {
        let (header, body) = self.make_header_body();
        Message {
            header,
            kind: MessageKind::Request(body),
        }
    }
}

impl MessageBuilder<Response, Yes, Yes> {
    pub fn build(self) -> Message {
        let (header, body) = self.make_header_body();
        Message {
            header,
            kind: MessageKind::Response(body),
        }
    }
}

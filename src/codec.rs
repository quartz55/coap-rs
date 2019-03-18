use crate::error::{Error as CoapError, MessageError};
use crate::message::error::FormatError;
use crate::message::{Header, Message};
use bytes::{BufMut, BytesMut};

pub enum Msg {
    Valid(Message),
    Invalid(Header, FormatError),
}
pub struct CoapCodec;
impl tokio::codec::Decoder for CoapCodec {
    type Item = Msg;
    type Error = CoapError;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match Message::from_bytes(&buf) {
            Ok(msg) => {
                buf.clear();
                Ok(Some(Msg::Valid(msg)))
            }
            Err(MessageError::PacketTooSmall(_)) => Ok(None),
            Err(MessageError::MessageFormat(err)) => {
                buf.truncate(4);
                let header = Header::from_bytes(&buf)?;
                buf.clear();
                Ok(Some(Msg::Invalid(header, err)))
            }
        }
    }
}
impl tokio::codec::Encoder for CoapCodec {
    type Item = Message;
    type Error = CoapError;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = msg.as_bytes()?;
        dst.put(bytes);
        Ok(())
    }
}

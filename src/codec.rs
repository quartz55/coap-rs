use crate::error::{Error as CoapError, MessageError};
use crate::message::error::{ErrorKind, FormatError};
use crate::message::{Header, Message};
use bytes::{BufMut, BytesMut};

pub enum Incoming {
    Valid(Message),
    Reject(Header, FormatError),
    Invalid(FormatError),
}

pub struct CoapCodec;

impl tokio::codec::Decoder for CoapCodec {
    type Item = Incoming;
    type Error = CoapError;
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match Message::from_bytes(&buf) {
            Ok(msg) => {
                buf.clear();
                Ok(Some(Incoming::Valid(msg)))
            }
            Err(err) => match (err.kind(), err.header()) {
                (ErrorKind::PacketTooSmall(_), _) => Ok(None),
                (ErrorKind::MessageFormat(err), Some(header)) => {
                    buf.clear();
                    Ok(Some(Incoming::Reject(header.clone(), err.clone())))
                }
                (ErrorKind::MessageFormat(err), None) => {
                    buf.clear();
                    Ok(Some(Incoming::Invalid(err.clone())))
                }
            },
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

use crate::message::error::Error as MessageError;

#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    Message(MessageError),
}

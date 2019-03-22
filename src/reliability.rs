use crate::message::header::MessageType;

#[derive(Debug, Clone, Copy)]
pub enum Reliablity {
    Confirmable,
    NonConfirmable,
}

impl Reliablity {
    pub fn from_message_type(message_type: MessageType) -> Option<Self> {
        match message_type {
            MessageType::Confirmable => Some(Reliablity::Confirmable),
            MessageType::NonConfirmable => Some(Reliablity::NonConfirmable),
            _ => None,
        }
    }
}

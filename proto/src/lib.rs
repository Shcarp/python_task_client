use message::Body;

pub mod message;

pub enum MessageType {
    PUSH,
    REQUEST,
    RESPONSE,
    OTHER,
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::PUSH => b'1',
            MessageType::REQUEST => b'2',
            MessageType::RESPONSE => b'3',
            MessageType::OTHER => b'0',
        }
    }
}

impl MessageType {
    pub fn from_u8(byte: u8) -> MessageType {
        match byte {
            b'1' =>MessageType::PUSH,
            b'2' => MessageType::REQUEST,
            b'3' => MessageType::RESPONSE,
            _ => MessageType::OTHER,
        }
    }
}

impl serde::Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        self.value.serialize(serializer)
    }
}


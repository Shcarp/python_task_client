use message::{Body, DataType, Push};
use serde::ser::SerializeStruct;
use serde_json::Value;
use protobuf::EnumOrUnknown;

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

impl serde::Serialize for Push {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        let mut state = serializer.serialize_struct("Push", 3)?;
        state.serialize_field("event", &self.event)?;
        state.serialize_field("status", &self.status.unwrap().value())?;
        state.serialize_field("data", &self.data.json_value())?;
        state.end()
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

impl DataType {
    pub fn from_i32(i: i32) -> DataType {
        match i {
            2 => DataType::Bool,
            0 => DataType::String,
            1 => DataType::Number,
            3 => DataType::Array,
            4 => DataType::Object,
            5 => DataType::Null,
            _ => DataType::Null,
        }
    }
}

impl Body {
    pub fn json_value(&self) -> Value {
        match self.value.parse::<Value>() {
            Ok(value) => {
                value
            },
            Err(_) => {
                Value::Null
            },
        } 
    }
}


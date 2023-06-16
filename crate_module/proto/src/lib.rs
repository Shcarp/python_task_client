use message::{Body, DataType, Push};
use serde::ser::SerializeStruct;
use serde_json::Value;

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
        state.serialize_field("sendTime", &self.sendTime)?;
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
        match self.type_ {
            Some(type_) => {
                match type_.unwrap() {
                    DataType::Bool => Value::Bool(serde_json::from_str(&self.value).unwrap()),
                    DataType::String => Value::String(self.value.to_string()),
                    DataType::Number => Value::Number(serde_json::from_str(&self.value).unwrap()),
                    DataType::Array => Value::Array(serde_json::from_str(&self.value).unwrap()),
                    DataType::Object => Value::Object(serde_json::from_str(&self.value).unwrap()),
                    DataType::Null => Value::Null,
                }
            },
            None => Value::Null,
        } 
    }
}


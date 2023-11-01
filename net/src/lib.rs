use serde::{Deserialize, Serialize};
use bincode::Error as BincodeError;

#[derive(Serialize, Deserialize, Debug)]

pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
}

impl MessageType {
    pub fn serialize(&self) -> Result<Vec<u8>, BincodeError> {
        bincode::serialize(&self)
    }
    pub fn deserialize(from: &[u8]) -> Result<Self, BincodeError> {
        bincode::deserialize(&from)
    }
}
use serde::{Deserialize, Serialize};
use bincode::Error as BincodeError;

#[derive(Serialize, Deserialize, Debug)]

pub enum Message {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, BincodeError> {
        bincode::serialize(&self)
    }
    pub fn deserialize(from: &[u8]) -> Result<Self, BincodeError> {
        bincode::deserialize(&from)
    }
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

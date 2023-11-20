use bincode::Error as BincodeError;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]

pub enum Message {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
    ClientQuit(String),
}

pub const STREAM_READ_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(thiserror::Error, Debug)]
pub enum ReceiveMessageError {
    #[error("Connection error")]
    StreamError(#[from] std::io::Error),
    #[error("Client disconnected")]
    ClientDisconnected(#[source] std::io::Error),
    #[error("Unable to deserialize message")]
    DeserializationError(#[from] BincodeError),
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, BincodeError> {
        bincode::serialize(&self)
    }
    pub fn deserialize(from: &[u8]) -> Result<Self, BincodeError> {
        bincode::deserialize(&from)
    }

    pub fn send_to(&self, tcp_stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let data = self.serialize()?;
        let data_len = data.len() as u32;
        tcp_stream.write(&data_len.to_be_bytes())?;
        tcp_stream.write_all(&data)?;
        Ok(())
    }

    // try to read the message from TcpStream
    // looks like tricky stuff because of the timeouts
    // I don't want to block the thread if there is nothing to do - that's why I seet timeout for initial lenght read; but after that if we have a len, let's wait forever
    // but ... what if the client is really slow when sending the message?
    // and part of the bytes (the length) is sent, but the rest is not?
    pub fn receive(stream: &mut TcpStream) -> Result<Option<Message>, ReceiveMessageError> {
        use ReceiveMessageError::*;
        let timeout_original = stream.read_timeout().unwrap_or(None);
        stream.set_read_timeout(Some(STREAM_READ_TIMEOUT))?;
        let data_len = {
            let mut len_bytes = [0u8; 4];
            let read_exact_result = stream.read_exact(&mut len_bytes);
            stream.set_read_timeout(timeout_original)?;
            match read_exact_result {
                Ok(_) => u32::from_be_bytes(len_bytes) as usize,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::TimedOut => return Ok(None), //timeout
                        std::io::ErrorKind::Interrupted => return Ok(None), //timeout   - takhle je to v dokumentaci read_exact; ale ve skutecnosti hazi TimedOut
                        std::io::ErrorKind::UnexpectedEof => return Err(ClientDisconnected(e)), //client disconnected
                        _ => {
                            //println!("{:?}, kind: {}", e, e.kind());
                            return Err(ClientDisconnected(e));
                        }
                    }
                }
            }
        };
        // note: not sure how to return error properly
        // I'd like to have error that contains information about connection error (e.g. client disconnected) as well about the deserialization error
        // enum?
        let message = {
            let mut buffer = vec![0u8; data_len];
            stream.read_exact(&mut buffer)?;
            Message::deserialize(&buffer)?
        };
        Ok(Some(message))
    }
}


pub mod logging {
    use env_logger::Env;
    use env_logger::init_from_env;
    
    pub fn init() {
        init_from_env(Env::default().default_filter_or("info"));
    }
}
use bincode::Error as BincodeError;
use log::{warn,debug};
use serde::{Deserialize, Serialize};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use std::error::Error;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, PartialEq)]

pub enum Message {
    Text { from: String, content: String },
    Image { from: String, content: Vec<u8> },
    File { from: String, name: String, content: Vec<u8> },
    ClientHello { from: String },
    ServerHello,
    ClientQuit { from: String },
}

pub const STREAM_READ_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(thiserror::Error, Debug)]
pub enum ReceiveMessageError {
    #[error("General stream error")]
    GeneralStreamError(#[from] std::io::Error),
    #[error("Client disconnected")]
    RemoteDisconnected(#[source] std::io::Error),
    #[error("Unable to deserialize message")]
    DeserializationError(#[from] BincodeError),
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, BincodeError> {
        let mut res = bincode::serialize(&self)?;
        if chaos::is_time_for_random_error() {
            res.remove(0);
            warn!("Chaos monkey is removing first byte from serialized message");
        }
        Ok(res)
    }
    pub fn deserialize(mut from: &[u8]) -> Result<Self, BincodeError> {
        if chaos::is_time_for_random_error() {
            warn!("Chaos monkey is removing first byte from message before deserialization");
            from = &from[1..];
        }
        bincode::deserialize(&from)
    }

    pub async fn send_async(&self, tcp_stream: &mut OwnedWriteHalf) -> Result<(), Box<dyn Error>> {
        let data = self.serialize()?;
        let data_len = data.len() as u32;
        tcp_stream.write(&data_len.to_be_bytes()).await?;
        tcp_stream.write_all(&data).await?;
        Ok(())
    }

    pub async fn receive_async(stream: &mut OwnedReadHalf) -> Result<Message, ReceiveMessageError> {

        use ReceiveMessageError::*;
        
        debug!("reading data len");
        let data_len = {
            let mut len_bytes = [0u8; 4];
            match stream.read_exact(&mut len_bytes).await
            {
                Ok(_) => u32::from_be_bytes(len_bytes) as usize,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => 
                { 
                    debug!("error reading data: {:?}", e); 
                    return Err(RemoteDisconnected(e))
                },
                Err(e) => { 
                    debug!("error reading data: {:?}", e); 
                    return Err(GeneralStreamError(e))
                }
            }
        };
        debug!("data len: {}", data_len);

        let message = {
            let mut buffer = vec![0u8; data_len];
            stream.read_exact(&mut buffer).await?;
            Message::deserialize(&buffer)?
        };
        Ok(message)
    }
}


pub mod logging {
    use env_logger::Env;
    use env_logger::init_from_env;
    
    pub fn init() {
        init_from_env(Env::default().default_filter_or("info"));
    }
}

pub mod chaos {

    pub fn enabled() -> bool {
        std::env::var("CHAOS_MONKEY").is_ok()
    }
    pub fn is_time_for_random_error() -> bool {
        enabled() && rand::random::<u8>() < 30
    }
}
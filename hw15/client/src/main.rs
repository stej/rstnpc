use shared::{Message, chaos, ReceiveMessageError};
use tokio::net::tcp::OwnedWriteHalf; //https://github.com/Miosso/rust-workspace
use std::fs;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::path::Path;
//use std::sync::mpsc;
//use std::sync::mpsc::Sender;
use std::time::Duration;
use std::time::SystemTime;
use clap::Parser;
use log::{info, debug, warn, error};
use anyhow::{Result, Context};

use tokio::sync::mpsc::{Sender, Receiver, channel};

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ConnectionArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
}

async fn process_stdin_command(command: &str, tcpstream: &mut OwnedWriteHalf) -> Result<(), Box<dyn std::error::Error>> {
    fn file_to_message(file_path: &str) -> Result<Message> {
        let path = Path::new(file_path);
        let content = fs::read(path)?;
        Ok(Message::File {
            name: path.file_name().context("Unable to get file name")?.to_str().context("Unable to get file name")?.into(),
            content,
        })
    }

    fn image_to_message(file_path: &str) -> Result<Message> {
        let Message::File { name: _, content } = 
            file_to_message(file_path).context("Image processing failed")?
            else {
                panic!("Unexpected type");
            };
        
        let path = Path::new(file_path);
        match path
            .extension()
            .context("Unable to get extension")?
            .to_str()
            .context("Unable to get extension")?
        {
            ".png" => Ok(Message::Image(content)),
            _ => Ok(Message::Image(content)),
        }
    }

    let command = command.trim();
    let message = if command.starts_with(".file ") {
        file_to_message(&command[".file ".len()..])?
    } else if command.starts_with(".image ") {
        image_to_message(&command[".image ".len()..])?
    } else {
        Message::Text(command.into())
    };

    debug!("-> {:?}", message);
    message.send_to_async(tcpstream).await
}

fn handle_message(message: &Message) {
    fn save_general_file(
        name: &str,
        content: &[u8],
        directory: &str,
    ) -> Result<()> {
        let dir = Path::new(directory);
        if !dir.exists() {
            fs::create_dir(dir)?;
        }
        let file_path = dir.join(name);
        fs::write(file_path, content)?;
        Ok(())
    }
    fn save_file(name: &str, content: &[u8]) -> Result<()> {
        save_general_file(name, content, "files")
    }
    fn save_img(content: &[u8]) -> Result<()> {
        let name = format!(
            "{}.png",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis()
                .to_string()
        );
        save_general_file(&name, content, "images")
    }
    let message_result = match message {
        Message::File { name, content } => {
            println!("Receiving {}", name);
            save_file(name, content)
        }
        Message::Image(content) => {
            println!("Receiving image...");
            save_img(content)
        }
        Message::Text(text) => {
            println!("{}", text);
            Ok(())
        }
        _ => Ok(()),
    };
    if let Err(e) = message_result {
        error!("{}", e);
    }
}

fn process_incomming_message_from_server(message: &Result<Message, ReceiveMessageError>) -> bool {
    use shared::ReceiveMessageError::*;

    match message {
        Ok(m) => handle_message(&m),
        Err(GeneralStreamError(e)) => { 
            error!("Server stream problems. Error: {}", e);
        },
        Err(DeserializationError(e)) => { 
            error!("Server sent malformed message. Error: {}", e);
        },
        Err(RemoteDisconnected(e)) => { 
            error!("Server disconnected. Error: {}", e);
        },
    }
    !message.is_err()
}

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<()> {
    shared::logging::init();
    if chaos::enabled() {
        warn!("Chaos monkey is enabled");
    }

    let args = ConnectionArgs::parse();
    info!("Connecting to {}:{}", args.host, args.port);

    let addr = SocketAddr::new(args.host.parse().unwrap(), args.port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();
    let remote = stream.local_addr().unwrap().to_string();
    info!("Connected as {}", remote);

    let (mut stream_reader, mut stream_writer) = stream.into_split();

    let mut rx_stdin = async_stdin::recv_from_stdin(1);
    loop {
        tokio::select!(
            Some(command) = rx_stdin.recv() => {
                let command = command.trim();
                if command == ".quit" {
                    break;
                }
                if let Err(e) = process_stdin_command(&command, &mut stream_writer).await {
                    error!("{}", e);
                }
            },
            message = Message::receive2_async(&mut stream_reader) => {
                if !process_incomming_message_from_server(&message) {
                    break;
                }
            }           
        )
    }
    Ok(())
}
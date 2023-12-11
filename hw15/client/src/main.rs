use shared::{Message, chaos, ReceiveMessageError};
use tokio::net::tcp::OwnedReadHalf;
use tokio::{net::tcp::OwnedWriteHalf, io::AsyncWriteExt}; //https://github.com/Miosso/rust-workspace
use tokio::io::AsyncReadExt;
use tokio::fs::File;
//use std::fs;
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::path::Path;
use std::time::SystemTime;
use clap::Parser;
use log::{info, debug, warn, error};
use anyhow::{Result, Context,anyhow};

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ConnectionArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
    #[arg(short = 'u', long, default_value = "")]
    user: String,
}

async fn process_stdin_command(user_name: &str, command: &str, tcpstream: &mut OwnedWriteHalf) -> Result<(), Box<dyn std::error::Error>> {
    async fn file_to_message(user_name: &str, file_path: &str) -> Result<Message> {
        let path = Path::new(file_path);
        let mut content = Vec::new();
        File::open(path)
            .await?
            .read_to_end(&mut content)
            .await?;
        Ok(Message::File {
            from: user_name.into(),
            name: path.file_name().context("Unable to get file name")?.to_str().context("Unable to get file name")?.into(),
            content,
        })
    }

    async fn image_to_message(user_name: &str, file_path: &str) -> Result<Message> {
        let Message::File { from:_, name: _, content } = 
            file_to_message(user_name, file_path).await.context("Image processing failed")?
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
            ".png" => Ok(Message::Image{ from: user_name.into(), content}),
            _ => Ok(Message::Image{ from: user_name.into(), content}),
        }
    }

    let command = command.trim();
    let message = if command.starts_with(".file ") {
        file_to_message(user_name, &command[".file ".len()..]).await?
    } else if command.starts_with(".image ") {
        image_to_message(user_name, &command[".image ".len()..]).await?
    } else {
        Message::Text{ from: user_name.into(), content: command.into() }
    };

    debug!("-> {:?}", message);
    message.send_async(tcpstream).await
}

async fn handle_message(current_user: &str, message: &Message) {
    async fn save_general_file(name: &str, content: &[u8], directory: &str) -> Result<()> {
        let dir = Path::new(directory);
        if !dir.exists() {
            tokio::fs::create_dir(dir).await?;
        }
        let file_path = dir.join(name);
        File::create(file_path)
            .await?
            .write_all(content)
            .await?;
        Ok(())
    }

    async fn save_file(name: &str, content: &[u8]) -> Result<()> {
        save_general_file(name, content, "files").await
    }

    async fn save_img(content: &[u8]) -> Result<()> {
        let name = format!(
            "{}.png",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis()
                .to_string()
        );
        save_general_file(&name, content, "images").await
    }
    let message_result = match message {
        Message::File { from, name, content } => {
            println!("|{}|[{}]: Receiving {}", current_user, from, name);
            save_file(name, content).await
        }
        Message::Image{ from, content} => {
            println!("|{}|[{}]: Receiving image...", current_user, from);
            save_img(content).await
        }
        Message::Text{ from, content} => {
            println!("|{}|[{}]: {}", current_user, from, content);
            Ok(())
        }
        Message::ClientHello { from } => {
            println!("|{}|[{}]: ...connected", current_user, from);
            Ok(())
        }
        Message::ClientQuit { from } => {
            println!("|{}|[{}]: ...disconnected", current_user, from);
            Ok(())
        },
        _ => {
            println!("|{}|Unexpected message: {:?}", current_user, message);
            Ok(())
        }
    };
    if let Err(e) = message_result {
        error!("{}", e);
    }
}

async fn process_incomming_message_from_server(current_user: &str, message: &Result<Message, ReceiveMessageError>) -> bool {
    use shared::ReceiveMessageError::*;

    match message {
        Ok(m) => handle_message(current_user, &m).await,
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

async fn try_send_hello(stream_reader: &mut OwnedReadHalf, stream_writer: &mut OwnedWriteHalf, user: &str) -> Result<()> {

    let msg = Message::ClientHello{ from: user.into() };

    // note: now idea how to just call
    //    msg.send_async(stream_writer).await?; 
    // so that it's converted to Result<()>
    // it complains: 
    //    `dyn std::error::Error` cannot be shared between threads safely
    //    the trait `Sync` is not implemented for `dyn std::error::Error` etc.
    // somewhere used anyhow::from_boxed (https://github.com/dtolnay/anyhow/issues/83), but it's obviously not possible anymore (anyhow::error::from_boxed is private)
    if let Err(e) = msg.send_async(stream_writer).await {
         return Err(anyhow!("Problems when sending hello message to server: {}", e));
    }
    if let Message::ServerHello = Message::receive_async(stream_reader).await? {
        info!("Connected as {}", user);
        Ok(())
    } else {
        Err(anyhow!("Unexpected message from server"))
    }
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

    let remote_addr = SocketAddr::new(args.host.parse()?, args.port);
    let stream = TcpStream::connect(&remote_addr).await?;
    let local_addr = stream.local_addr()?.to_string();

    let user = if args.user.is_empty() {  local_addr.clone()} 
                    else {args.user };
    let (mut stream_reader, mut stream_writer) = stream.into_split();
    info!("Connecting as {}, user {}", local_addr, user);
    if let Err(e) = try_send_hello(&mut stream_reader, &mut stream_writer, &user).await {
        info!("Server closed connection. {}", e);
        return Ok(());
    }

    let mut rx_stdin = async_stdin::recv_from_stdin(1);
    loop {
        tokio::select!(
            Some(command) = rx_stdin.recv() => {
                let command = command.trim();
                if command == ".quit" {
                    break;
                }
                if let Err(e) = process_stdin_command(&user, &command, &mut stream_writer).await {
                    error!("{}", e);
                }
            },
            message = Message::receive_async(&mut stream_reader) => {
                if !process_incomming_message_from_server(&user, &message).await {
                    break;
                }
            }           
        )
    }
    Ok(())
}
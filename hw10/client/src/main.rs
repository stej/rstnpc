use std::path::Path;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::error::Error;
use shared::Message;    //https://github.com/Miosso/rust-workspace
use std::fs;
use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
struct ConnectionArgs {
    #[arg(short, long, default_value="11111")]
    port: u16,
    #[arg(short='s', long, default_value="localhost")]
    host: String,
}

fn send_message(message: &Message, tcp_stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    // let serialized_message = message.serialize()?;
    // tcp_stream.write_all(&serialized_message)?;
    // Ok(())
    let data = message.serialize()?;
    let data_len = data.len() as u32;
    tcp_stream.write(& data_len.to_be_bytes())?;
    tcp_stream.write_all(&data)?;
    Ok(())
}

fn process_command(command: &str, tcp_stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    fn file_to_message(file_path: &str) -> Result<Message, Box<dyn Error>> {
        let path = Path::new(file_path);
        if path.exists() {
            let content = fs::read(path).unwrap();
            Ok(Message::File { name: file_path.into(), content })
        } else {
            Err(format!("File {} does not exist", file_path).into())
        }
    }

    fn image_to_message(file_path: &str) -> Result<Message, Box<dyn Error>> {
        let file = file_to_message(file_path)?;
        let Message::File{name: _, content} = file else {
            panic!("Expected file, but got {:?}", file);
        };
        let path = Path::new(file_path);
        match path.extension().ok_or("Unable to get extension")?.to_str().ok_or("Unable to get extension")? {
            ".png"  => Ok(Message::Image(content)),
            _  => Ok(Message::Image(content)),              // todo
        }
    }

    let command = command.trim();
    let message = 
        if command.starts_with(".file ") {
            file_to_message(&command.replace(".file ", ""))     // todo slice?
        } else if command.starts_with(".image ") {
            image_to_message(&command.replace(".image ", ""))   // todo slice?
        } else {
            Ok(Message::Text(command.into()))
        };
    match message {
        Ok(message) => { 
            println!("Sending message: {:?}", message);
            send_message(&message, tcp_stream)
        },
        Err(error) /*@ e*/ => Err(error)                            // todo zjednodusit?
    }
}

fn main() {
    let args = ConnectionArgs::parse();
    println!("Connecting to {}:{}", args.host, args.port);

    let addr = SocketAddr::new(args.host.parse().unwrap(), args.port);
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(30)).unwrap();
    // let buf = b"Hello, world!";
    // stream.write_all(buf).unwrap();


    let stdin = std::io::stdin();
    loop {
        let mut line = String::new();
        let command_result = 
            match stdin.read_line(&mut line) {
                Ok(_) if line.trim() == ".quit" => break,
                Ok(_) => process_command(&line, &mut stream),
                Err(error) => Err(error.into()),
            };
        if let Err(e) = command_result {
            eprintln!("Error: {}", e);
        }
    }
}

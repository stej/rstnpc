use clap::Parser;
use std::{net::TcpListener, io::Read, error::Error, net::TcpStream};
use shared::Message;

#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long)]
    port: u16,
    #[arg(short='s', long)]
    host: String,
}

fn read_message(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let data_len = {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;
        u32::from_be_bytes(len_bytes) as usize
    };
    let message = {
        let mut buffer =  vec![0u8; data_len];
        stream.read_exact(&mut buffer)?;
        Message::deserialize(&buffer)?
    };
    println!("Received message: {:?}", message);
    Ok(())
}

fn main() {
    let args = ListenerArgs::parse();
    println!("Listening on {}:{}", args.host, args.port);

    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let addr = stream.peer_addr().unwrap();
        println!("Received connection from {}", addr);

        read_message(&mut stream).unwrap();
    }
}

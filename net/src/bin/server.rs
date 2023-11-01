use net::MessageType;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for connection in listener.incoming() {
        let mut connection = connection.unwrap();
        let mut len_bytes = [0u8; 4];
        connection.read_exact(&mut len_bytes).unwrap();

        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buffer = vec![0u8; len];
        connection.read_exact(&mut buffer).unwrap();

        let message = MessageType::deserialize(&buffer).unwrap();
        println!("{:?}", message);
    }
}

// cargo build --all-targets
// cargo run --bin server
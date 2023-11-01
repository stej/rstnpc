use clap::Parser;
use std::{net::TcpListener, io::Read, error::Error, net::TcpStream, net::SocketAddr};
use shared::Message;
use std::time::Duration;
use std::collections::HashMap;

#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value="11111")]
    port: u16,
    #[arg(short='s', long, default_value="localhost")]
    host: String,
}

#[derive(Debug)]
struct ConnectedClients {
    clients: HashMap<SocketAddr, TcpStream>
}

impl ConnectedClients { 
    fn add(&mut self, client: TcpStream) {
        self.clients.insert(client.peer_addr().unwrap(), client);
    }

    fn new() -> Self {
        Self { clients: HashMap::new() }
    }
}

fn read_message(stream: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
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
    Ok(message)
}

fn read_messages_from_clients(clients: &mut ConnectedClients) -> Vec<(Message, SocketAddr)> {
    let mut temp_buff = [0u8; 1];
    let mut received = Vec::new();
    for (&addr, client) in &mut clients.clients {
        let Ok(read_bytes) = client.peek(&mut temp_buff) else {
            continue;
        };
        if read_bytes > 0 {
            let msg = read_message(client);
            match msg {
                Ok(m) => received.push((m, addr)),
                Err(e) => eprintln!("Error reading message: {}", e)
            }
        }
    }
    received
}

fn accept_connection(stream: Result<TcpStream, std::io::Error>) -> Option<TcpStream> {
    match stream {
        Ok(s) => Some(s),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None, // no pending connections
        Err(e) => panic!("Encountered IO error: {}", e),
    }
}

fn broadcast_messages(clients: &ConnectedClients, incomming_messages: Vec<(Message, SocketAddr)>) {
    println!("{:?}", clients);
    println!("{:?}", incomming_messages);
}

fn main() {
    let args = ListenerArgs::parse();
    println!("Listening on {}:{}", args.host, args.port);

    let mut clients = ConnectedClients::new();
    let listener = {
        let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).unwrap();
        listener.set_nonblocking(true).expect("Unable to set non-blocking");
        listener
    };

    for possible_stream in listener.incoming() {
        if let Some(stream) = accept_connection(possible_stream) {
            let addr = stream.peer_addr().unwrap();
            println!("New connection from {}", addr);
            clients.add(stream);
        }
        let incomming_messages = read_messages_from_clients(&mut clients);
        broadcast_messages(&mut clients, incomming_messages);
        std::thread::sleep(Duration::from_millis(10));
    }
}

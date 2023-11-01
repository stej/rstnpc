use clap::Parser;
use std::{net::TcpListener, io::Read, error::Error, net::TcpStream};
use shared::Message;
use std::time::Duration;

#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value="11111")]
    port: u16,
    #[arg(short='s', long, default_value="localhost")]
    host: String,
}

struct ConnectedClients {
    clients: Vec<TcpStream>
}

impl ConnectedClients { 
    fn add(&mut self, client: TcpStream) {
        self.clients.push(client);
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

fn handle_incomming_connection(clients: &mut ConnectedClients, stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    println!("New connection from {}", addr);
    clients.add(stream);
}

fn read_messages_from_clients<'t>(clients: &'t mut ConnectedClients) -> Vec<(Message, &'t mut TcpStream)> {
    let mut temp_buff = [0u8; 1];
    let mut received = Vec::new();
    for client in &mut clients.clients {
        let Ok(read_bytes) = client.peek(&mut temp_buff) else {
            continue;
        };
        if read_bytes > 0 {
            let msg = read_message(client);
            match msg {
                Ok(m) => received.push((m, client)),
                Err(e) => eprintln!("Error reading message: {}", e)
            }
        }
    }
    received
}

fn accept_connections(clients: &mut ConnectedClients, stream: Result<TcpStream, std::io::Error>) {
    match stream {
        Ok(s) => handle_incomming_connection(clients, s),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => (), // no pending connections
        Err(e) => panic!("Encountered IO error: {}", e),
    }
}

fn broadcast_messages<'t>(clients: &'t mut ConnectedClients, incomming_messages: Vec<(Message, &mut TcpStream)>) {
    
}

fn main() {
    let args = ListenerArgs::parse();
    println!("Listening on {}:{}", args.host, args.port);

    let mut clients = ConnectedClients { clients: Vec::new() };
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).unwrap();
    listener.set_nonblocking(true).expect("Unable to set non-blocking");

    for stream in listener.incoming() {
        accept_connections(&mut clients, stream);
        let incomming_messages = read_messages_from_clients(&mut clients);
        broadcast_messages(&mut clients, incomming_messages);
        std::thread::sleep(Duration::from_millis(10));
    }
}

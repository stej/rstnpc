use clap::Parser;
use std::{net::TcpListener, net::TcpStream, net::SocketAddr};
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

    fn receive_messages(&mut self) -> Vec<(Message, SocketAddr)> {
        let mut received = Vec::new();
        for (&addr, client) in &mut self.clients {
            match Message::receive(client) {
                Ok(Some(m)) => { 
                    println!("+: {:?}", m);
                    received.push((m, addr))
                },
                Ok(None) => {},
                Err(e) => eprintln!("Error reading message: {}", e)
            }
        }
        received
    }

    fn remove(&mut self, clients_to_remove: &Vec<SocketAddr>) {
        clients_to_remove.iter().for_each(|addr| { 
            println!("Removing client {}", addr);
            self.clients.remove(&addr).unwrap();
        });
    }
}

fn accept_connection(stream: Result<TcpStream, std::io::Error>) -> Option<TcpStream> {
    match stream {
        Ok(s) => { 
            let addr = s.peer_addr().unwrap();
            println!("New connection from {}", addr);
            s.set_nonblocking(false)
                .expect("Unable to set non-blocking");
            s.set_read_timeout(
                Some(shared::STREAM_READ_TIMEOUT))
                .expect("Unable to set read timeout");
            Some(s) 
        },
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => None, // no pending connections
        Err(e) => panic!("Encountered IO error: {}", e),
    }
}
fn remove_dead_clients(clients: &mut ConnectedClients, incomming_messages: &Vec<(Message, SocketAddr)>) {
    let clients_to_remove = 
        incomming_messages.iter()
        .filter(|(m, _)| matches!(m, Message::ClientQuit(_)))
        .map(|(_, addr)| addr.clone())
        .collect();
    clients.remove(&clients_to_remove);
}

fn broadcast_messages(clients: &mut ConnectedClients, incomming_messages: Vec<(Message, SocketAddr)>) {
    if incomming_messages.len() == 0 {
        return
    }
    println!("clients : {:?}", clients.clients.keys());
    println!("messages: {:?}", incomming_messages);

    for (msg, message_origin_address) in incomming_messages {
        clients.clients
            .iter_mut()
            .filter(|(a, _)| **a != message_origin_address)
            .for_each(|(_, c)| {
                match msg.send_to(c) {
                    Ok(_) => {},
                    Err(e) => eprintln!("Error sending message: {}", e)
                }
            });
    }
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
            clients.add(stream);
        }
        let incomming_messages = clients.receive_messages();
        remove_dead_clients(&mut clients, &incomming_messages);
        broadcast_messages(&mut clients, incomming_messages);
        std::thread::sleep(Duration::from_millis(10));
    }
}

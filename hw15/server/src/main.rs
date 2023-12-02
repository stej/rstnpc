use clap::Parser;
use shared::{Message, chaos};
use std::collections::HashMap;
use std::time::Duration;
//use std::{net::SocketAddr, net::TcpListener, net::TcpStream};
use log::{info, debug, warn, error};
use shared::ReceiveMessageError::*;
use anyhow::{Result, Context};
use tokio::net::{TcpListener, TcpStream};

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
}

//#[derive(Debug)]
// struct ConnectedClients {
//     clients: HashMap<SocketAddr, TcpStream>,
// }

// impl ConnectedClients {
//     fn add(&mut self, client: TcpStream) {
//         self.clients.insert(client.peer_addr().unwrap(), client);
//     }

//     fn new() -> Self {
//         Self {
//             clients: HashMap::new(),
//         }
//     }

    // fn receive_messages(&mut self) -> Vec<(Message, SocketAddr)> {
    //     let mut received = Vec::new();
    //     for (&addr, client) in &mut self.clients {
    //         match Message::receive(client) {
    //             Ok(Some(m)) => {
    //                 received.push((m, addr));
    //             }
    //             Ok(None) => {}
    //             Err(RemoteDisconnected(e)) => {
    //                 info!("Client {} disconnected. Error: {}", addr, e);
    //                 received.push((Message::ClientQuit("".into()), addr))
    //             }
    //             Err(GeneralStreamError(e)) => { 
    //                 error!("Client {} stream problems. Error: {}", addr, e);
    //                 received.push((Message::ClientQuit("".into()), addr))
    //             },
    //             Err(DeserializationError(e)) => { 
    //                 error!("Client {} sent malformed message. Error: {}", addr, e);
    //             },
    //         }
    //     }
    //     received
    // }

    // fn remove(&mut self, clients_to_remove: &Vec<SocketAddr>) {
    //     clients_to_remove.iter().for_each(|addr| {
    //         info!("Removing client {}", addr);
    //         self.clients.remove(&addr).unwrap();
    //     });
    // }
//}

// fn remove_dead_clients(
//     clients: &mut ConnectedClients,
//     incomming_messages: &Vec<(Message, SocketAddr)>,
// ) {
//     let clients_to_remove = incomming_messages
//         .iter()
//         .filter(|(m, _)| matches!(m, Message::ClientQuit(_)))
//         .map(|(_, addr)| addr.clone())
//         .collect::<Vec<SocketAddr>>();
//     if clients_to_remove.is_empty() {
//         return;
//     }
//     info!("Removing clients: {:?}", clients_to_remove);
//     clients.remove(&clients_to_remove);
// }

// fn broadcast_messages(
//     clients: &mut ConnectedClients,
//     incomming_messages: Vec<(Message, SocketAddr)>,
// ) {
//     if incomming_messages.len() == 0 {
//         return;
//     }
//     debug!("clients : {:?}", clients.clients.keys());
//     debug!("messages: {:?}", incomming_messages);

//     for (msg, message_origin_address) in incomming_messages {
//         clients
//             .clients
//             .iter_mut()
//             .filter(|(a, _)| **a != message_origin_address)
//             .for_each(|(_, c)| match msg.send_to(c) {
//                 Ok(_) => {}
//                 Err(e) => error!("Error sending message: {}", e),
//             });
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    shared::logging::init();
    if chaos::enabled() {
        warn!("Chaos monkey is enabled");
    }

    let args = ListenerArgs::parse();
    info!("Listening on {}:{}", args.host, args.port);

    ///let mut clients = ConnectedClients::new();
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port))
                            .await
                            .context("Unable to create listener. Is there any other instance running?")?;

    let (msg_out, msg_in) = flume::unbounded();
    let (sock_out, sock_in) = flume::unbounded();
    // reader from channel with messages
    tokio::spawn(async move {
        use tokio::select;
        loop {
            let Ok(message) : Result<Message, _> = msg_in.recv_async().await else {
                return;
            };
            println!("{:?}", message);
        }
    });

    loop {
        let (mut stream, addr) = match listener.accept().await {
            Ok((stream, addr)) => {
                let addr = stream.peer_addr().unwrap();
                info!("New connection from {}", addr);
                (stream, addr)
            }
            Err(e) => { 
                error!("Encountered IO error: {}. Skipping the new connection attempt.", e);
                continue;
            }
        };


        let tx = msg_out.clone();
        //clients.add(stream);
        tokio::spawn(async move {
            async fn send(tx: &flume::Sender<shared::Message>, message: Message) {
                if let Err(e) = tx.send_async(message).await {
                    error!("Error sending message: {}", e);
                }
            }
            loop {
                match Message::receive_async(&mut stream).await {
                    Ok(message) => send(&tx, message).await,
                    Err(e) => println!("Error: {}", e)
                }
            }
        });

        // let incomming_messages = clients.receive_messages();
        // remove_dead_clients(&mut clients, &incomming_messages);
        // broadcast_messages(&mut clients, incomming_messages);
        // std::thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

use clap::Parser;
use shared::{Message, chaos};
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use std::collections::HashMap;
use std::net::SocketAddr;
use log::{info, debug, warn, error};
use shared::ReceiveMessageError::*;
use anyhow::{Result, Context};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{channel as mpscchannel, Sender, Receiver};
use tokio::select;

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
}

#[derive(Debug)]
struct ConnectedClients {
    clients: HashMap<SocketAddr, OwnedWriteHalf>,
}

impl ConnectedClients {
    fn add(&mut self, client: OwnedWriteHalf) {
        debug!("New socket: {:?}", client);
        self.clients.insert(client.peer_addr().unwrap(), client);
    }

    fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    fn remove(&mut self, client_to_remove: &SocketAddr) {
        debug!("all clients: {:?}", self.clients.keys());
        debug!("client to remove: {:?}", client_to_remove);
        match self.clients.remove(client_to_remove) {
            Some(_) => (),
            None => debug!("Client {} already removed.", client_to_remove),
        }

        if self.clients.is_empty() {
            info!("No clients connected.");
        }
    }

    async fn broadcast_message(&mut self, incomming_message: (Message, SocketAddr)) {
        debug!("all clients : {:?}", self.clients.keys());
        info!("message: {:?}", incomming_message);

        let (msg, message_origin_address) = incomming_message;

        for (socket_addr,write_stream) in self.clients.iter_mut() {
            if *socket_addr != message_origin_address {
                match msg.send_async(write_stream).await {
                        Ok(_) => { info!("  ... sent to {:?}", socket_addr); },
                        Err(e) => error!("Error sending message: {}", e),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    shared::logging::init();
    if chaos::enabled() {
        warn!("Chaos monkey is enabled");
    }

    let args = ListenerArgs::parse();
    info!("Listening on {}:{}", args.host, args.port);

    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port))
                            .await
                            .context("Unable to create listener. Is there any other instance running?")?;

    let (tx_msg, rx_msg) = mpscchannel::<(SocketAddr, Message)>(1024);
    let (tx_sock, rx_sock) = mpscchannel::<tokio::net::tcp::OwnedWriteHalf>(1024);

    // one global task that receives messages from (a) clients, (b) with new connections
    spawn_task_holding_connected_clients(rx_sock, rx_msg);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {}", addr);

                let (stream_reader, stream_writer) = stream.into_split();

                // register new client; it's stored with other clients so that it's possible to broadcast the incomming message
                tx_sock.send(stream_writer).await.unwrap();
                
                let tx_msg = tx_msg.clone();
                spawn_new_task_handling_one_client(stream_reader, tx_msg);
            }
            Err(e) => { 
                error!("Encountered IO error: {}. Skipping the new connection attempt.", e);
                continue;
            }
        };
    }
}

/// task that holds all connected clients and broadcasts messages to them
/// 
/// the clients (TCP streams) are stored in local variable
/// incomming data come from two channels:
/// - `rx_sock` - writeable streams to register for broadcasing
/// - `rx_msg` - message to broadcast (that arrived from any client)
fn spawn_task_holding_connected_clients(mut rx_sock: Receiver<OwnedWriteHalf>, mut rx_msg: Receiver<(SocketAddr, Message)>) {
    tokio::spawn(async move {
        let mut clients = ConnectedClients::new();

        loop {
            select! {
                Some((socket_addr, message)) = rx_msg.recv() => {
                    debug!("Message from channel {:?}: {:?}", socket_addr, message);

                    match message {
                        Message::ClientQuit(_) => clients.remove(&socket_addr),
                        _ => clients.broadcast_message((message, socket_addr)).await,
                    }
                }
                Some(sock) = rx_sock.recv() => {
                    clients.add(sock);
                }
            }
        }
    });
}

/// task that handles one client
/// 
/// the task is using read part of the TCP stream to receive messages from the client
/// the message is decoded and sent to the channel `tx_msg` to be broadcasted to other clients
fn spawn_new_task_handling_one_client(mut stream: OwnedReadHalf, tx_msg: Sender<(SocketAddr, shared::Message)>)  {
    tokio::spawn(async move {
        async fn send(tx: &Sender<(SocketAddr, shared::Message)>, message: (SocketAddr, Message)) {
            if let Err(e) = tx.send(message).await {
                error!("Error sending message: {}", e);
            }
        }
        let stream_addr = stream.peer_addr().unwrap();
        loop {
            match Message::receive_async(&mut stream).await {
                Ok(message) => send(&tx_msg, (stream_addr, message)).await,
                Err(GeneralStreamError(e)) => { 
                    error!("Client {} stream problems. Error: {}. Exitting...", stream_addr, e);
                    send(&tx_msg, (stream_addr, Message::ClientQuit(stream_addr.to_string()))).await;
                    break;
                },
                Err(RemoteDisconnected(e)) => { 
                    error!("Client {} disconnected. Error: {}. Exitting...", stream_addr, e);
                    send(&tx_msg, (stream_addr, Message::ClientQuit(stream_addr.to_string()))).await;
                    break;
                },
                Err(DeserializationError(e)) => { 
                    error!("Client {} sent malformed message. Error: {}", stream_addr, e);
                },
            }
        }
    });
}

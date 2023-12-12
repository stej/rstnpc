mod db;
mod connected_clients;

use clap::Parser;
use shared::{Message, chaos};
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use log::{info, debug, warn, error};
use shared::ReceiveMessageError::*;
use anyhow::{Result, Context};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel as mpscchannel, Sender, Receiver};
use tokio::select;
use connected_clients::{ConnectedClient, ConnectedClients};

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
}



struct IncommingClientMessage {
    user_name: String,
    message: Message,
}

#[tokio::main]
async fn main() -> Result<()> {
    shared::logging::init();
    if chaos::enabled() {
        warn!("Chaos monkey is enabled");
    }

    db::ensure_db_exists().await?;

    let args = ListenerArgs::parse();
    info!("Listening on {}:{}", args.host, args.port);

    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port))
                            .await
                            .context("Unable to create listener. Is there any other instance running?")?;

    let (tx_msg, rx_msg) = mpscchannel::<IncommingClientMessage>(1024);
    let (tx_sock, rx_sock) = mpscchannel::<ConnectedClient>(1024);
    let (tx_client_disconnect, mut rx_client_disconnect) = mpscchannel::<String>(1);

    // one global task that receives (a) chat messages from clients, (b) new connections
    spawn_task_holding_connected_clients(rx_sock, rx_msg, tx_client_disconnect);

    let mut connected_users = Vec::new();
    loop {
        select!(
            new_client = listener.accept() => match new_client {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
    
                    let Some((user_name, stream_reader, stream_writer)) = try_process_new_user(stream, &connected_users).await else {
                        continue;
                    };
                    connected_users.push(user_name.to_string());
                    // register new client; it's stored with other clients so that it's possible to broadcast the incomming message
                    tx_sock.send(connected_clients::ConnectedClient{user_name: user_name.to_string(), stream_writer}).await.unwrap();
                    
                    let tx_msg = tx_msg.clone();
                    spawn_new_task_handling_one_client(user_name, stream_reader, tx_msg);
                }
                Err(e) => { 
                    error!("Encountered IO error: {}. Skipping the new connection attempt.", e);
                    continue;
                }
            },
            Some(user_name) = rx_client_disconnect.recv() => {
                connected_users.retain(|u| u != &user_name);
            }
        )
    }
}

// makes first contact with client and checks whether the client can be connected
//
// the client can not be connected if there is any other already connected client with the same name
async fn try_process_new_user(stream: TcpStream, connected_users: &Vec<String>) -> Option<(String, OwnedReadHalf, OwnedWriteHalf)> {

    // checks whether the user that is trying to register on server, can be connected
    async fn try_user_handshake(stream_reader: &mut OwnedReadHalf, stream_writer: &mut OwnedWriteHalf, already_connected_users: &Vec<String>) -> Result<Option<String>>  {
        let hello_message = Message::receive(stream_reader).await;
        let Ok(Message::ClientHello {from: user }) =  hello_message else  {
            error!("Unexpected message from client: {:?}", hello_message);
            return Ok(None)
        };
        if already_connected_users.contains(&user) {
            error!("User {} already connected", user);
            return Ok(None)
        }
        match Message::ServerHello.send(stream_writer).await {
            Ok(()) => Ok(Some(user)),
            Err(e) => { error!("Error when sending server hello: {}", e); Ok(None)}, // convert to anyhow??
        }
    }

    let (mut stream_reader, mut stream_writer) = stream.into_split();
    match try_user_handshake(&mut stream_reader, &mut stream_writer, &connected_users).await {
        Ok(Some(user_name)) => Some((user_name, stream_reader, stream_writer)),
        _ => None,
    }
}

/// task that holds all connected clients and broadcasts messages to them
/// 
/// the clients (TCP streams) are stored in local variable
/// incomming data come from two channels:
/// - `rx_sock` - writeable streams to register for broadcasing
/// - `rx_msg` - message to broadcast (that arrived from any client)
/// - `tx_client_disconnect` - notification channel back to main thread that the client has disconnected
/// 
/// there is also some DB logic
/// - user presence is updated after each incomming message
/// - all incomming messages are stored in db
/// - if the client is offline, the messages are stored in db and sent to the client when it connects again
fn spawn_task_holding_connected_clients(mut rx_sock: Receiver<ConnectedClient>, mut rx_msg: Receiver<IncommingClientMessage>, tx_client_disconnect: Sender<String>) {

    tokio::spawn(async move {
        let mut clients = ConnectedClients::new();

        loop {
            select! {
                Some(IncommingClientMessage{user_name, message}) = rx_msg.recv() => {
                    debug!("Message from channel {:?}: {:?}", user_name, message);
                    
                    if matches!(message, Message::ClientQuit{from:_}) {
                        clients.remove(&user_name);
                        tx_client_disconnect.send(user_name.clone()).await.unwrap();
                    } 

                    match message {
                        Message::Text{ .. } | 
                        Message::Image { .. } | 
                        Message::File { .. } => db::store_message(&user_name, &message).await,
                        _ => {}
                    };

                    clients.broadcast_message((message, user_name)).await;

                    db::update_online_users(&clients.get_clients()).await;
                },
                Some(mut client) = rx_sock.recv() => {
                    for msg in db::get_missing_messages(&client.user_name).await {
                        msg.send(&mut client.stream_writer).await.unwrap();
                    }
                    clients.add(client);
                }
            }
        }
    });
}

/// task that handles one client
/// 
/// the task is using read part of the TCP stream to receive messages from the client
/// the message is decoded and sent to the channel `tx_msg` to be broadcasted to other clients
fn spawn_new_task_handling_one_client(user_name: String, mut stream: OwnedReadHalf, tx_msg: Sender<IncommingClientMessage>)  {
    tokio::spawn(async move {

        async fn send(tx: &Sender<IncommingClientMessage>, user_name: &str, message: Message) {
            let msg = IncommingClientMessage { user_name: user_name.to_string(), message };
            if let Err(e) = tx.send(msg).await {
                error!("Error sending message: {}", e);
            }
        }

        // send "hello" message to other clients
        send(&tx_msg, &user_name, Message::ClientHello{ from: user_name.to_string() }).await;

        // process other incomming messages
        loop {
            match Message::receive(&mut stream).await {
                Ok(message) => send(&tx_msg, &user_name, message).await,
                Err(GeneralStreamError(e)) => { 
                    error!("Client {} stream problems. Error: {}. Exitting...", user_name, e);
                    send(&tx_msg, &user_name, Message::ClientQuit{from: user_name.to_string()}).await;
                    break;
                },
                Err(RemoteDisconnected(e)) => { 
                    error!("Client {} disconnected. Error: {}. Exitting...", user_name, e);
                    send(&tx_msg, &user_name, Message::ClientQuit{from: user_name.to_string()}).await;
                    break;
                },
                Err(DeserializationError(e)) => { 
                    error!("Client {} sent malformed message. Error: {}", user_name, e);
                },
            }
        }
    });
}
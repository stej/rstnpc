#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_include_static_resources;

mod db;
mod actor_connected_clients;
mod actor_db;
mod web;

use clap::Parser;
use shared::{Message, chaos};
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use log::{info, warn, error};
use shared::ReceiveMessageError::*;
use anyhow::{Result, Context};
use tokio::net::{TcpListener, TcpStream};
//use tokio::sync::mpsc::{channel as mpscchannel, Sender, Receiver};
//use tokio::select;
//use actor_connected_clients::{ConnectedClientMessage, ConnectedClients, IncommingClientMessage};
use ractor::{Actor, ActorRef};
use actor_connected_clients::ConnectedClientsActorMessage;

// looks like common code for client and server, but this is not typical dry sample
#[derive(Parser)]
struct ListenerArgs {
    #[arg(short, long, default_value = "11111")]
    port: u16,
    #[arg(short = 's', long, default_value = "localhost")]
    host: String,
}

#[rocket::main]
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

    let (db_actor, db_actor_handle) = 
        Actor::spawn(Some("actor_db".to_string()), actor_db::DbAccessActor, ())
            .await
            .expect("Failed to start actor with access to db");

    let (connected_cli_actor, connected_cli_actor_handle) = 
        Actor::spawn(Some("actor_clients".to_string()), actor_connected_clients::ConnectedClientsActor{db: db_actor.clone()}, ())
            .await
            .expect("Failed to start actor with connected clients");

    tokio::spawn(async move {
        web::rocket(db_actor.clone()).launch().await.unwrap();
        info!("Web server has exited..")
    });
                                                            
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {}", addr);

                let Some((user_name, stream_reader, stream_writer)) = try_process_new_user(stream, &connected_cli_actor).await else {
                    continue;
                };

                // register new client; it's stored with other clients so that it's possible to broadcast the incomming message
                connected_cli_actor.cast(ConnectedClientsActorMessage::NewClient{user_name: user_name.to_string(), stream_writer}).unwrap();
                
                spawn_new_task_handling_one_client(user_name, stream_reader, connected_cli_actor.clone());
            }
            Err(e) => { 
                error!("Encountered IO error: {}. Skipping the new connection attempt.", e);
                continue;
            }
        }
    }
}

// makes first contact with client and checks whether the client can be connected
//
// the client can not be connected if there is any other already connected client with the same name
async fn try_process_new_user(stream: TcpStream, actor: &ActorRef<ConnectedClientsActorMessage>) -> Option<(String, OwnedReadHalf, OwnedWriteHalf)> {

    // checks whether the user that is trying to register on server, can be connected
    async fn try_user_handshake(stream_reader: &mut OwnedReadHalf, stream_writer: &mut OwnedWriteHalf, actor: &ActorRef<ConnectedClientsActorMessage>) -> Result<Option<String>>  {
        let hello_message = Message::receive(stream_reader).await;
        let Ok(Message::ClientHello {from: user }) =  hello_message else  {
            error!("Unexpected message from client: {:?}", hello_message);
            return Ok(None)
        };
        let can_connect = ractor::call!(actor, ConnectedClientsActorMessage::CheckUserCanConnect, user.to_string()).expect("Failed to check whether user can connect");
        if !can_connect {
            error!("User {} already connected", user);
            return Ok(None)
        }
        match Message::ServerHello.send(stream_writer).await {
            Ok(()) => Ok(Some(user)),
            Err(e) => { error!("Error when sending server hello: {}", e); Ok(None)}, // convert to anyhow??
        }
    }

    let (mut stream_reader, mut stream_writer) = stream.into_split();
    match try_user_handshake(&mut stream_reader, &mut stream_writer, actor).await {
        Ok(Some(user_name)) => Some((user_name, stream_reader, stream_writer)),
        _ => None,
    }
}

/// task that handles one client
/// 
/// the task is using read part of the TCP stream to receive messages from the client
/// the message is decoded and sent to the channel `tx_msg` to be broadcasted to other clients
fn spawn_new_task_handling_one_client(user_name: String, mut stream: OwnedReadHalf, actor: ActorRef<ConnectedClientsActorMessage>)  {
    tokio::spawn(async move {

        fn send(actor: &ActorRef<ConnectedClientsActorMessage>, user_name: &str, message: Message) {
            let msg = ConnectedClientsActorMessage::IncommingChatMessage { user_name: user_name.to_string(), message };
            if let Err(e) = actor.cast(msg) {
                error!("Error sending message: {}", e);
            }
        }

        // send "hello" message to other clients
        send(&actor, &user_name, Message::ClientHello{ from: user_name.to_string() });

        // process other incomming messages
        loop {
            match Message::receive(&mut stream).await {
                Ok(message) => send(&actor, &user_name, message),
                Err(GeneralStreamError(e)) => { 
                    error!("Client {} stream problems. Error: {}. Exitting...", user_name, e);
                    send(&actor, &user_name, Message::ClientQuit{from: user_name.to_string()});
                    break;
                },
                Err(RemoteDisconnected(e)) => { 
                    error!("Client {} disconnected. Error: {}. Exitting...", user_name, e);
                    send(&actor, &user_name, Message::ClientQuit{from: user_name.to_string()});
                    break;
                },
                Err(DeserializationError(e)) => { 
                    error!("Client {} sent malformed message. Error: {}", user_name, e);
                },
            }
        }
    });
}
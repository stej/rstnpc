use log::{error, info, debug};
use shared::Message;
use std::collections::HashMap;
use tokio::net::tcp::OwnedWriteHalf;
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use tokio::sync::mpsc::{channel as mpscchannel, Sender, Receiver};
use tokio::select;
use crate::db;

#[derive(Debug)]
pub struct ConnectedClients {
    clients: HashMap<String, OwnedWriteHalf>,
}

impl ConnectedClients {
    pub fn add(&mut self, user_name: String, stream_writer: OwnedWriteHalf) {
        debug!("New client: {:?}", user_name);
        self.clients.insert(user_name, stream_writer);
    }

    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    pub fn remove(&mut self, client_to_remove: &str) {
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

    pub async fn broadcast_message(&mut self, incomming_message: (Message, String)) {
        debug!("all clients : {:?}", self.clients.keys());
        info!("message: {:?}", incomming_message);

        let (msg, message_origin_client) = incomming_message;

        for (client,write_stream) in self.clients.iter_mut() {
            if *client != message_origin_client {
                match msg.send(write_stream).await {
                        Ok(_) => { info!("  ... sent to {:?}", client); },
                        Err(e) => error!("Error sending message: {}", e),
                }
            }
        }
    }

    pub fn get_clients(&self) -> Vec<String> {
        self.clients.keys()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }
}


pub struct ConnectedClientsActor;

pub struct IncommingClientMessage {
    pub user_name: String,
    pub message: Message,
}

pub struct ConnectedClientMessage {
    pub user_name: String,
    pub stream_writer: OwnedWriteHalf
}

pub enum ConnectedClientsActorMessage
{
    IncommingChatMessage {
        user_name: String,
        message: Message,
    },
    NewClient {
        user_name: String,
        stream_writer: OwnedWriteHalf
    },
    CheckUserCanConnect(String, RpcReplyPort<bool>),    // todo: struct?
    GetClientsCount(RpcReplyPort<usize>),
}

#[async_trait]
impl Actor for ConnectedClientsActor {
    type Msg = ConnectedClientsActorMessage;
    type State = ConnectedClients;
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _: ()) -> Result<Self::State, ActorProcessingErr> {
        let clients = ConnectedClients::new();
        Ok(clients)
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, clients: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message {
            ConnectedClientsActorMessage::GetClientsCount(reply) => {
                if reply.send(clients.clients.len()).is_err() {
                    error!("Error sending reply");
                }
            },
            ConnectedClientsActorMessage::IncommingChatMessage { user_name, message } => {
                debug!("Message from channel {:?}: {:?}", user_name, message);
                    
                    if matches!(message, Message::ClientQuit{from:_}) {
                        clients.remove(&user_name);
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
            ConnectedClientsActorMessage::NewClient { user_name, mut stream_writer } => {
                for msg in db::get_missing_messages(&user_name).await {
                    msg.send(&mut stream_writer).await.unwrap();
                }
                clients.add(user_name, stream_writer);
            },
            ConnectedClientsActorMessage::CheckUserCanConnect(user_name, reply ) => {
                let already_connected = clients.clients.contains_key(&user_name);
                if reply.send(!already_connected).is_err() {
                    error!("Error sending reply");
                }
            },
        }
        Ok(())
    }    
}
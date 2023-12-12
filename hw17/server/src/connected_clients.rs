use log::{error, info, debug};
use shared::Message;
use std::collections::HashMap;
use tokio::net::tcp::OwnedWriteHalf;

pub struct ConnectedClient {
    pub user_name: String,
    pub stream_writer: OwnedWriteHalf
}

#[derive(Debug)]
pub struct ConnectedClients {
    clients: HashMap<String, OwnedWriteHalf>,
}

impl ConnectedClients {
    pub fn add(&mut self, client: ConnectedClient) {
        debug!("New client: {:?}", client.user_name);
        self.clients.insert(client.user_name, client.stream_writer);
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
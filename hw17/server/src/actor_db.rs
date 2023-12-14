use crate::db;
use shared::Message;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

pub struct DbAccessActor;

pub enum DbMessage {
    StoreChatMessage{ user_name: String, message: Message },
    UpdateLastSeen{ user_names: Vec<String> },
    //GetMissingChatMessageSinceLastSeen{ user_name: String, reply: RpcReplyPort<Vec<Message>> },
    GetMissingChatMessageSinceLastSeen(String, RpcReplyPort<Vec<Message>>)
}

#[async_trait]
impl Actor for DbAccessActor {
    type Msg = DbMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _: ()) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, clients: &mut Self::State) -> Result<(), ActorProcessingErr> {
        match message {
            DbMessage::StoreChatMessage{user_name, message} => {
                db::store_message(&user_name, &message).await
            },
            // DbMessage::GetMissingChatMessageSinceLastSeen { user_name, reply } => {
            DbMessage::GetMissingChatMessageSinceLastSeen(user_name, reply) => {
                let messages = db::get_missing_messages(&user_name).await;
                if reply.send(messages).is_err() {
                    error!("Error sending reply");
                }
            },
            DbMessage::UpdateLastSeen { user_names} => {
                db::update_online_users(&user_names).await;
            }
        }
        Ok(())
    }
}
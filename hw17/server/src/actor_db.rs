use crate::db;
use shared::Message;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

pub struct DbAccessActor;

pub struct UserData {
    pub user_name: String,
    pub last_seen: std::time::SystemTime,
}

pub struct StoredMessage {
    pub user_name: String,
    pub time: std::time::SystemTime,
    pub message: Message
}

pub enum DbMessage {
    StoreChatMessage{ user_name: String, message: Message },
    UpdateLastSeen{ user_names: Vec<String> },
    //GetMissingChatMessageSinceLastSeen{ user_name: String, reply: RpcReplyPort<Vec<Message>> },
    GetMissingChatMessageSinceLastSeen(String, RpcReplyPort<Vec<Message>>),
    GetAllUsersLastSeen(RpcReplyPort<Vec<UserData>>),
    ListAllMessages(Option<String>, RpcReplyPort<Vec<StoredMessage>>),
    ForgetUser { user_name: String},
}

#[async_trait]
impl Actor for DbAccessActor {
    type Msg = DbMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(&self, _myself: ActorRef<Self::Msg>, _: ()) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(&self, _myself: ActorRef<Self::Msg>, message: Self::Msg, _: &mut Self::State) -> Result<(), ActorProcessingErr> {
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
            },
            DbMessage::GetAllUsersLastSeen(reply) => {
                let user_data = 
                    db::get_all_last_online_data().await
                    .into_iter()
                    .map(|(user, time)| UserData { user_name: user, last_seen: time})
                    .collect();
                if reply.send(user_data).is_err() {
                    error!("Error sending reply");
                }
            },
            DbMessage::ListAllMessages(user, reply) => {
                let messages = db::get_all_messages(user).await
                    .into_iter()
                    .map(|(time, user_name, message)| StoredMessage {time, user_name, message })
                    .collect();
                if reply.send(messages).is_err() {
                    error!("Error sending reply with messages");
                }
            },
            DbMessage::ForgetUser { user_name } => {
                db::forget_user(user_name).await;
            }
        }
        Ok(())
    }
}
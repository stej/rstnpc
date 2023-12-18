use rocket::http::Status;
use rocket::response::{content, status, Redirect};
use rocket::{Rocket, Request, Build, State, serde};

use crate::actor_db;
use ractor::ActorRef;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

use rocket_dyn_templates::Template;
use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};

fn  format_time(time: SystemTime) -> String {
    let datetime: DateTime<Utc> = time.into();
    datetime.format("%Y-%m-%d %T").to_string()
}

#[get("/<code>")]
fn forced_error(code: u16) -> Status {
    Status::new(code)
}

#[catch(404)]
fn general_not_found() -> content::RawHtml<&'static str> {
    content::RawHtml(r#"
        <p>Hmm... What are you looking for?</p>
        Say <a href="/hello/Sergio/100">hello!</a>
    "#)
}

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
    let msg = format!("{} ({})", status, req.uri());
    status::Custom(status, msg)
}

use serde::Serialize;
#[get("/users")]
async fn users(state: &State<ActorRef<actor_db::DbMessage>>) -> Template {

    let Ok(cli) = ractor::call!(state, actor_db::DbMessage::GetAllUsersLastSeen) else {
        return Template::render("error", &HashMap::from([("error", "Unable to get users")]));
    };
    let data = cli.into_iter()
                        .map(|r| (r.user_name, format_time(r.last_seen)))
                        .collect::<Vec<_>>();
    info!("Returning users: {:?}", data);

    #[derive(Serialize)]
    struct Data {
        users: Vec<(String, String)>,
        rendered: String
    }
    let datax = Data { users: data, rendered: format_time(std::time::SystemTime::now()) };
    Template::render("users", &datax)
}

#[post("/users/delete/<user>")]
async fn delete_user(user: &str, state: &State<ActorRef<actor_db::DbMessage>>) -> Redirect {

    let Ok(()) = state.cast(actor_db::DbMessage::ForgetUser{user_name: user.into()}) else {
        error!("Error when deleting user.");
        return rocket::response::Redirect::to(uri!(users));
    };
    rocket::response::Redirect::to(uri!(users))
}

#[get("/messages?<user>")]
async fn messages(user: Option<String>, state: &State<ActorRef<actor_db::DbMessage>>) -> Template {
    let Ok(messages) = ractor::call!(state, actor_db::DbMessage::ListAllMessages, user) else {
        return Template::render("error", &HashMap::from([("error", "Unable to get messages")]));
    };
    
    #[derive(Serialize)]
    struct TemplateMessage {
        user: String,
        time: String,
        kind: String,
        data: String,
    }
    #[derive(Serialize)]
    struct Data {
        messages: Vec<TemplateMessage>,
        rendered: String,
    }

    let messages = 
        messages.into_iter()
        .map(|row| {
            let (kind, data) = match row.message {
                shared::Message::Text { content, .. } => ("t".to_string(), content.to_string()),
                shared::Message::Image { content, .. } => ("i".to_string(), general_purpose::STANDARD.encode(&content)),
                shared::Message::File { name, .. } => ("f".to_string(), name.to_string()),
                _ => ("".to_string(),"".to_string()),
            };
            TemplateMessage { user: row.user_name,  time: format_time(row.time), kind, data }
        })
        .collect();
    let data = Data { rendered: format_time(std::time::SystemTime::now()), messages };
    Template::render("messages", &data)
}

#[get("/")]
async fn index() -> Redirect {
    rocket::response::Redirect::to(uri!(users))
}

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
    "/images/disk.png" => disk_png => "disk",
    "/images/textbubble.png" => textbubble_png => "tbubble",
}

pub fn rocket(db_actor: ActorRef<actor_db::DbMessage>) -> Rocket<Build> {

    rocket::build()
        .mount("/", routes![index, users, delete_user, messages, forced_error])
        .manage(db_actor)
        .attach(Template::custom(|_engines| {
            //engines.handlebars.register_helper("simple-helper", Box::new(web_handlebars_ext::SimpleHelper));
        }))
        .attach(static_resources_initializer!(
            "favicon" => "images/favicon.ico",
            "disk" => "images/disk.png",
            "tbubble" => "images/comment-text.png"
        ))
        .mount("/", routes![favicon, disk_png, textbubble_png])
        .register("/", catchers![general_not_found, default_catcher])
}
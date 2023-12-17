use rocket::http::Status;
use rocket::response::{content, status, Redirect};
use rocket::{Rocket, Request, Build, State, serde};

use crate::actor_db;
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use chrono::{DateTime, Utc};
use std::time::SystemTime;

use rocket_dyn_templates::Template;
use std::collections::HashMap;

fn  format_time(time: SystemTime) -> String {
    // let time: DateTime<Utc> = time.into(); 
    // time.to_rfc3339()
    let datetime: DateTime<Utc> = time.into();
    datetime.format("%Y-%m-%d %T").to_string()
}

#[get("/hello/<name>/<age>")]
async fn hello(state: &State<ActorRef<actor_db::DbMessage>>, name: &str, age: i8) -> String {
    let add = match ractor::call!(state, actor_db::DbMessage::GetAllUsersLastSeen) {
        Ok(cli) => cli.into_iter()
                        .map(|r| format!("{}: {:?}<br>", r.user_name, format_time(r.last_seen)))
                        .collect::<Vec<String>>()
                        .join(","),
        Err(_) => "error...".into()
    };
    format!("Hello, {} year old named {}! Res: {}", age, name, add)
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
    let data = messages.into_iter()
                        .map(|r| (format_time(r.time), r.user_name, format!("{:?}", r.message)))
                        .collect::<Vec<_>>();
    info!("Returning messages: {:?}", data);
    
    #[derive(Serialize)]
    struct Data {
        messages: Vec<(String, String, String)>,
        rendered: String
    }
    let data = Data { messages: data, rendered: format_time(std::time::SystemTime::now()) };
    Template::render("messages", &data)
}

pub fn rocket(db_actor: ActorRef<actor_db::DbMessage>) -> Rocket<Build> {

    let handlebars = handlebars::Handlebars::new();

    rocket::build()
        .mount("/", routes![hello, users, delete_user, messages, forced_error])
        .attach(Template::fairing())
        .manage(db_actor)
        .manage(handlebars)
        .register("/", catchers![general_not_found, default_catcher])
        //.register("/hello", catchers![hello_not_found])
        //.register("/hello/Sergio", catchers![sergio_error])
}

// fn rocket() -> Rocket<Build> {
//     rocket::build()
//         // .mount("/", routes![hello, hello]) // uncomment this to get an error
//         // .mount("/", routes![unmanaged]) // uncomment this to get a sentinel error
//         .mount("/", routes![hello, forced_error])
//         .register("/", catchers![general_not_found, default_catcher])
//         .register("/hello", catchers![hello_not_found])
//         .register("/hello/Sergio", catchers![sergio_error])
// }

// #[rocket::main]
// async fn main() {
//     if let Err(e) = rocket().launch().await {
//         println!("Whoops! Rocket didn't launch!");
//         // We drop the error to get a Rocket-formatted panic.
//         drop(e);
//     };
// }

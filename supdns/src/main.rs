#[macro_use] extern crate rocket;
mod dns_txt_resolver;
mod spf_checker;

use spf_checker::{Checker, SpfToCheck, SpfCheckResult, create_checker};
use std::sync::mpsc;
use rocket::serde::json::{Json};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
struct MyErr {
    // workaround problem 'the trait `Sized` is not implemented for `(dyn StdError + 'static)`'
    //https://stackoverflow.com/a/62452516/75224
    error: String
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/check_spf_str/<domain>")]
fn check_spf_str(domain: &str) -> String {
    let (remote_sender, receiver) = mpsc::channel::<SpfCheckResult>();

    //let a = create_checker();
    CHECKER.get_sender().send(SpfToCheck{domain: domain.to_string(), sender_back: remote_sender}).expect("should be ok");
    let SpfCheckResult{result, records} = receiver.recv().unwrap();
    format!("<html><body>{} -> {}\\br{}</body></html>", domain, result, records.join("\\br"))
}

#[get("/check_spf/<domain>")]
fn check_spf(domain: &str) -> Json<Result<SpfCheckResult, MyErr>> {
    let (remote_sender, receiver) = mpsc::channel::<SpfCheckResult>();
    CHECKER.get_sender()
        .send(SpfToCheck{domain: domain.to_string(), sender_back: remote_sender})
        .expect("should be ok");
    let recv_result = receiver.recv();
    match recv_result {
        Ok(spf_result) => Json(Ok(spf_result)),
        Err(e) => Json(Err(MyErr{error: e.to_string()}))
    }
}

use once_cell::sync::Lazy;
static CHECKER: Lazy<Checker> = Lazy::new(|| create_checker());

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![check_spf])
}
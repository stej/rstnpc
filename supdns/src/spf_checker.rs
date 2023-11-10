//mod dns_txt_resolver;

use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use std::thread::{spawn, sleep};
use std::time::Duration;
use crate::dns_txt_resolver;
use rocket::serde::{Serialize, Deserialize};



#[derive(Debug)]
pub struct SpfToCheck { 
    pub domain: String,
    pub sender_back: Sender<SpfCheckResult>
}


#[derive(Serialize, Deserialize)]
pub struct SpfCheckResult {
    pub result: bool,
    pub records: Vec<String>,
}

pub struct Checker {
    sender: mpsc::Sender<SpfToCheck>,
    //receiver: mpsc::Receiver<SpfToCheck>
}

impl Checker {
    fn process_request(domain: &str, sender_back: &Sender<SpfCheckResult>) {
        println!("Got: {:?}", domain);
        let spf_records = 
            dns_txt_resolver::resolve_record(&domain)
            .into_iter()
            .filter(|record| record.starts_with("v=spf1"))
            .collect::<Vec<String>>();
        println!("spf_records: {:?}", spf_records);
        sender_back
            .send(SpfCheckResult{result: !spf_records.is_empty(), records: spf_records})
            .expect("should be ok");
    }
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<SpfToCheck>();
        spawn(move || {
            loop {
                match rx.recv() {
                    Ok(SpfToCheck{domain, sender_back}) => { 
                        Self::process_request(&domain, &sender_back)
                    },
                    Err(e) => eprintln!("err: {}", e)
                }
                sleep(Duration::from_millis(500));
                //println!("{:?}", rx);
            }
        });
        Checker{sender: tx}
    }

    pub fn get_sender(&self) -> mpsc::Sender<SpfToCheck> {
        self.sender.clone()
    }
}

pub fn create_checker() -> Checker {
    println!("create_checker");
    Checker::new()
}
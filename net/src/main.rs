use serde::{Deserialize, Serialize};
use net::MessageType;


#[derive(Serialize, Debug, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

use std::{net::TcpStream, io::Write};

fn main() {
    let point = Point { x: 1, y: 2 };
    let serialized = serde_json::to_string(&point).unwrap();
    println!("{}", serialized);
    let point: Point = serde_json::from_str(&serialized).unwrap();
    println!("{:?}", point);

    let point = Point { x: 1, y: 2 };
    let serialized = ron::to_string(&point).unwrap();
    println!("{}", serialized);
    let point: Point = ron::from_str(&serialized).unwrap();
    println!("{:?}", point);

    // let point = Point { x: 1, y: 2 };
    // let serialized = bson::to_string(&point).unwrap();
    // println!("{}", serialized);
    // let point: Point = bson::from_str(&serialized).unwrap();
    // println!("{:?}", point);

    let message = MessageType::Text("Hello World".to_string());
    let serialized = message.serialize().unwrap();

    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let len = serialized.len() as u32;
    stream.write(&len.to_be_bytes()).unwrap();
    stream.write_all(&serialized).unwrap();
}

/*
workspace - vice crate dohromady
1 crate, tam main a adresar bin/ tam server.js
2. mmkdir workspace
  cd workspace
  cargo new library --lib
  cargo new server
  cargo new client
  cargo build --all-targets
cargo.toml
        [workspace]
        members = ["library", "server", "client"]
        resolver = "2" .. proc?
        [dependencies]
        serde = { workspace=true, version = "1.0", features = ["derive"] }
*/

// server - prijmout connection, prijmenout message, broadcast vsem ostatnim
// nechci blokovat pri prijimani message
// nastavit set_nonblocking; 18:40
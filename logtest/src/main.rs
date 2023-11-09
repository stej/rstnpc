use simple_logger::SimpleLogger;
use log::{info, warn, as_serde};

struct Point {
    x: i32,
    y: i32,
}



fn main() {
    SimpleLogger::new().init().unwrap();
    let l = SimpleLogger::new();
    l.init();

    info!(target: "main", "Hello, world!");
    warn!("Hello, warn!");

    // let p = Point { x: 1, y: 2 };
    // info!(target: "yak_events", yak = as_serde!(p); "Commencing yak shaving");
}

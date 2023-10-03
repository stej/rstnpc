use std::{thread, time::Duration};

fn main() {
    ctrlc::set_handler(move || {
        println!("Unstopable!!");
    })
    .expect("Unable to run forever");

    for _ in 1..  {
        println!("Hello, world!");
        // looks like sleeping is needed to get the ctrl-c to work
        thread::sleep(Duration::from_millis(5));
    }
    
}

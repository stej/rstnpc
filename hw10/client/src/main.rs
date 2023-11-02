use core::panic;
use std::path::Path;
use std::net::{SocketAddr, TcpStream};
use std::error::Error;
use shared::Message;    //https://github.com/Miosso/rust-workspace
use std::fs;
use std::time::Duration;
use std::time::SystemTime;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use clap::Parser;

#[derive(Parser)]
struct ConnectionArgs {
    #[arg(short, long, default_value="11111")]
    port: u16,
    #[arg(short='s', long, default_value="localhost")]
    host: String,
}

fn process_stdin_command(command: &str, tx: &Sender<Message>) -> Result<(), Box<dyn Error>> {
    fn file_to_message(file_path: &str) -> Result<Message, Box<dyn Error>> {
        let path = Path::new(file_path);
        if path.exists() {
            let content = fs::read(path).unwrap();
            Ok(Message::File { name: path.file_name().unwrap().to_str().unwrap().into(), content })
        } else {
            Err(format!("File {} does not exist", file_path).into())
        }
    }

    fn image_to_message(file_path: &str) -> Result<Message, Box<dyn Error>> {
        let file = file_to_message(file_path)?;
        let Message::File{name: _, content} = file else {
            panic!("Expected file, but got {:?}", file);
        };
        let path = Path::new(file_path);
        match path.extension().ok_or("Unable to get extension")?.to_str().ok_or("Unable to get extension")? {
            ".png"  => Ok(Message::Image(content)),
            _  => Ok(Message::Image(content)),              // todo convert?
        }
    }

    let command = command.trim();
    let message = 
        if command.starts_with(".file ") {
            file_to_message(&command[".file ".len()..])
        } else if command.starts_with(".image ") {
            image_to_message(&command[".image ".len()..])
        } else {
            Ok(Message::Text(command.into()))
        };
    match message {
        Ok(message) => { 
            println!("-> {:?}", message);
            tx.send(message).map_err(|e| e.into())
        },
        Err(error) /*@ e*/ => Err(error)                            // todo zjednodusit?
    }
}

fn handle_message(message: &Message) {
    fn save_general_file(name: &str, content: &[u8], directory: &str) -> Result<(), Box<dyn Error>> {
        let dir = Path::new(directory);
        if !dir.exists() { 
            fs::create_dir(dir)?;
        }
        let file_path = dir.join(name);
        fs::write(file_path, content)?;
        Ok(())
    }
    fn save_file(name: &str, content: &[u8]) -> Result<(), Box<dyn Error>> {
        save_general_file(name, content, "files")
    }
    fn save_img(content: &[u8]) -> Result<(), Box<dyn Error>> {
        let name = format!("{}.png", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis().to_string());
        save_general_file(&name, content, "images")
    }
    match message {
        Message::File { name, content } => { 
            println!("Receiving {}", name);
            save_file(name, content).unwrap()                                                 // todo 
        },       
        Message::Image(content) => {                                                // todo                
            println!("Receiving image...");    
            save_img(content).unwrap()
        },
        Message::Text(text) => println!("{}", text),
        _ => ()
    }
}

fn try_receive_message(stream: &mut TcpStream) {
    match Message::receive(stream) {
        Ok(Some(m)) => { 
            handle_message(&m)
        },
        Ok(None) => {},
        Err(e) =>  {
            // this would be great to have a reason of the error - e.g. server disconnected
            // match e.kind() { 
            //     std::io::ErrorKind::ConnectionAborted | 
            //     std::io::ErrorKind::ConnectionReset |
            //     std::io::ErrorKind::ConnectionRefused => {
            //         panic!("Connection closed by server");
            //     },
            //     _ => eprintln!("Error reading message: {}", e)
            // }
            eprintln!("Error reading message: {}", e)
        }
    }
}

fn main() {
    let args = ConnectionArgs::parse();
    println!("Connecting to {}:{}", args.host, args.port);

    let addr = SocketAddr::new(args.host.parse().unwrap(), args.port);
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(30)).unwrap();
    let remote = stream.local_addr().unwrap().to_string();

    let (tx, rx) = mpsc::channel::<shared::Message>();
    let stream_commander = std::thread::spawn(move || {
        loop {
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(message) => {
                    message.send_to(&mut stream).expect("Unable to send message");
                    if matches!(message, Message::ClientQuit(_)) {
                        break;
                    }
                },
                Err(_) => {
                    // nothing to send, try to receive message
                    try_receive_message(&mut stream)
                }
            }
        }
        println!("Exiting...");
    });
 
    let stdin = std::io::stdin();
    loop {
        let mut line = String::new();
        let command_result = 
            match stdin.read_line(&mut line) {
                Ok(_) if line.trim() == ".quit" => break,
                Ok(_) => process_stdin_command(&line, &tx),
                Err(error) => Err(error.into()),
            };
        if let Err(e) = command_result {
            eprintln!("Error: {}", e);
        }
    }
    tx.send(Message::ClientQuit(remote)).unwrap();
    stream_commander.join().unwrap();
}

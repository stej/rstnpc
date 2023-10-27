mod csv;
mod operation;

use regex::Regex;
use std::{process, env};
use std::io::{Read, stdin};
use std::error::Error;
use operation::{Operation, OperationWithParam};
use std::thread;
use std::fs::read_to_string;
use std::path::Path;


static OPTIONS: &str = "lowercase|uppercase|slugify|no-spaces|len|reverse|csv";

fn usage() {
    println!("Usage: <app> <{}>", OPTIONS);
    process::exit(1);
}

fn read_input() -> Result<String, Box<dyn Error>> {
    println!("Specify some input: ");
    let mut input =  Vec::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_end(&mut input)?;
    
    Ok(input.into_iter().map(|c| c as char).collect::<String>())
}

fn lowercase(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.to_lowercase())
}
fn uppercase(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.to_uppercase())
}
fn slugify(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(slug::slugify(input))
}
fn no_space(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", ""))
}
fn len(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.len().to_string())
}
fn reverse(input: &str) -> Result<String, Box<dyn Error>> {
    Ok(input.chars().rev().collect::<String>())
}
fn csv(input: &str) -> Result<String, Box<dyn Error>> {
    let csv = csv::Csv::parse(input)?;
    Ok(csv.to_string())
}

fn split_line_to_operation_and_arg(line: &str) -> Result<OperationWithParam, Box<dyn Error>> {
    if line.trim().is_empty() {
        return Ok(OperationWithParam{operation: Operation::Exit, param: String::new()});
    }

    let mut split = line.splitn(2, ' ');
    let operation =
        split
        .next().ok_or("No operation found")?
        .parse::<Operation>()?;
    let arg = split
        .next()
        .ok_or("No argument found")?;
    Ok(OperationWithParam{operation, param: arg.trim().to_string()})
}


fn main() {
    let args: Vec<String> = env::args().collect();

    // if args.len() > 1 {
    //     if args.len() != 2 {
    //         usage();
    //     }
    //     let option = args[1].as_str();
    //     let args_regex = Regex::new(format!(r"^({})$", OPTIONS).as_str()).unwrap();
    //     if !args_regex.is_match(option) {
    //         usage();
    //     }
        
    //     let result =
    //         match option {
    //             "lowercase" => lowercase(),
    //             "uppercase" => uppercase(),
    //             "slugify" => slugify(),
    //             "no-spaces" => no_space(),
    //             "len" => len(),
    //             "reverse" => reverse(),
    //             "csv" => csv(),
    //             _ => panic!("Unknown option")
    //         };
    //     return match result {
    //         Ok(transmuted) => println!("{}", transmuted),
    //         Err(err) => eprintln!("Operation '{}' failed: {}", option, err)
    //     };
    // }

    let (send, rec) = std::sync::mpsc::channel();
    let input_thread = thread::spawn(move || {
        let mut line = String::new();
        loop {
            stdin().read_line(&mut line).unwrap();
            let what_to_do = split_line_to_operation_and_arg(&line);
            match what_to_do {
                Ok(op) 
                    if op.operation == Operation::Exit => { 
                        send.send(op).unwrap();
                        println!("Exiting sender.");
                        break;
                    }
                Ok(op) => send.send(op).unwrap(),
                Err(err) => eprintln!("Error: {}", err)
            }
            line.clear()
        }
    });
    let process_thread = thread::spawn(move || {
        loop {
            let message = rec.recv();
            let Ok(OperationWithParam { operation, param }) = 
                message else {
                    panic!("Unexpected input: {:?}", message);
                };
            let result =
                match operation {
                    Operation::Lowercase => lowercase(&param),
                    Operation::Uppercase => uppercase(&param),
                    Operation::Slugify => slugify(&param),
                    Operation::NoSpaces => no_space(&param),
                    Operation::Len => len(&param),
                    Operation::Reverse => reverse(&param),
                    Operation::Csv => { 
                        let path = Path::new(&param);
                        let file_content = read_to_string(&path).expect("Unable to read file");
                        csv(&file_content)
                    },
                    Operation::Exit => {
                        println!("Exiting receiver..");
                        break
                    }
                };
            match result {
                Ok(transmuted) => println!("{}", transmuted),
                Err(err) => eprintln!("Operation '{:?}' failed: {}", operation, err)
            };
        }
    });
    input_thread.join().unwrap();
    process_thread.join().unwrap();
}
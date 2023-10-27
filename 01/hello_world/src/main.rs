mod csv;
mod operation;

use std::{env};
use std::io::{Read, stdin};
use std::error::Error;
use operation::{Operation, OperationWithParam};
use std::thread::{self};
use std::fs::read_to_string;
use std::path::Path;
use std::sync::mpsc::Sender;

/// Reads input from stdin until EOF (CTRL+D / CTRL+Z).
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

/// Expected to get string like `<operation> <argument>`
/// 
/// Returns `OperationWithParam` or error if the input is not in the expected format.
/// 
fn split_line_to_operation_and_arg(line: &str) -> Result<OperationWithParam, Box<dyn Error>> {
    if line.trim().is_empty() {
        return Ok(OperationWithParam::exit());
    }

    let mut split = line.splitn(2, ' ');
    let operation =
        split
        .next().ok_or("No operation found")?
        .parse::<Operation>()?;
    let arg = split
        .next()
        .ok_or("No argument found")?;
    Ok(OperationWithParam::from_interactive(operation, arg.trim().to_string()))
}

/// Used when the program is called with a command line argument.
/// Only first argument is used. It's expected to be operation name.
/// The rest of the input is read from stdin.
fn handle_from_cmdline(cmdline_arg: &str, send: Sender<OperationWithParam>) {
    match cmdline_arg.parse::<Operation>() {
        Ok(operation) => {
            let input = read_input().expect("Unable to read stdin");
            send.send(OperationWithParam::from_cmdline(operation, input)).unwrap();
        }
        Err(err) => eprintln!("Error: {}", err)
    }
    send.send(OperationWithParam::exit()).unwrap();
}


fn main() {

    let (send, rec) = std::sync::mpsc::channel();
    
    // start always; will process even the commands from cmdline
    // one way of doing things
    // although thre is of course the possibility to branch the code more
    let process_thread = thread::spawn(move || {
        fn process_csv_message(read_from_file: bool, operation_param: String) -> Result<String, Box<dyn Error>> {
            let file_content = 
                if read_from_file {
                    let path = Path::new(&operation_param);
                    read_to_string(&path)
                        .map_err(|err| format!("Unable to read file: {}", err))?
                } else {
                    operation_param
                };
            csv(&file_content)
        }

        loop {
            let message = rec.recv();
            let Ok(OperationWithParam { operation, param, is_interactive_operation }) = message else {
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
                    Operation::Csv => process_csv_message(is_interactive_operation, param),
                    Operation::Exit => {
                        //println!("Exiting receiver..");
                        break
                    }
                };
            match result {
                Ok(transmuted) => println!("{}", transmuted),
                Err(err) => eprintln!("Operation '{:?}' failed: {}", operation, err)
            };
        }
    });

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        handle_from_cmdline(&args[1], send);
    } else {
        println!("Type empty line to exit.");
        thread::spawn(move || {
            let mut line = String::new();
            loop {
                stdin().read_line(&mut line).unwrap();
                let what_to_do = split_line_to_operation_and_arg(&line);
                match what_to_do {
                    Ok(op) =>  {
                        let is_exit = op.operation == Operation::Exit;
                        send.send(op).unwrap();
                        if is_exit { 
                            //println!("Exiting sender.");
                            break;
                        }
                    }
                    Err(err) => eprintln!("Error: {}", err)
                }
                line.clear()
            }
        }).join().unwrap();
    };

    process_thread.join().unwrap();
}
mod csv;

use regex::Regex;
use std::{process, env};
use std::io::Read;
use std::error::Error;

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

fn lowercase() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(input.trim().to_lowercase())
}
fn uppercase() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(input.trim().to_uppercase())
}
fn slugify() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(slug::slugify(input.trim()))
}
fn no_space() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(input.trim().replace(" ", ""))
}
fn len() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(input.trim().len().to_string())
}
fn reverse() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    Ok(input.trim().chars().rev().collect::<String>())
}
fn csv() -> Result<String, Box<dyn Error>> {
    let input = read_input()?;
    let csv = csv::Csv::parse(input.trim())?;
    Ok(csv.to_string())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
    }
    let option = args[1].as_str();
    let args_regex = Regex::new(format!(r"^({})$", OPTIONS).as_str()).unwrap();
    if !args_regex.is_match(option) {
        usage();
    }
    
    let result =
        match option {
            "lowercase" => lowercase(),
            "uppercase" => uppercase(),
            "slugify" => slugify(),
            "no-spaces" => no_space(),
            "len" => len(),
            "reverse" => reverse(),
            "csv" => csv(),
            _ => panic!("Unknown option")
        };
    match result {
        Ok(transmuted) => println!("{}", transmuted),
        Err(err) => eprintln!("Operation '{}' failed: {}", option, err)
    }
}

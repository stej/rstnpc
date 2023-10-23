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

fn read_input() -> String {
    println!("Specify some input: ");
    let mut input =  Vec::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_end(&mut input).unwrap();    // todo: handle error
    input.into_iter().map(|c| c as char).collect::<String>()
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
    
    let input = read_input();
    let input = input.trim();

    let result =
        match option {
            "lowercase" => lowercase(input),
            "uppercase" => uppercase(input),
            "slugify" => slugify(input),
            "no-spaces" => no_space(input),
            "len" => len(input),
            "reverse" => reverse(input),
            "csv" => csv(input),
            _ => panic!("Unknown option")
        };
    match result {
        Ok(transmuted) => println!("{}", transmuted),
        Err(err) => eprintln!("Operation '{}' failed: {}", option, err)
    }
}

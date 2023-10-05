use regex::Regex;
use std::{io, process, env};
use slug::slugify;

static OPTIONS: &str = "lowercase|uppercase|slugify|no-spaces|len|reverse";

fn usage() {
    println!("Usage: <app> <{}>", OPTIONS);
    process::exit(1);
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
    

    // not possible.. ? compiler is complaining..
    // let input = ({
    //     println!("Specify some input: ");
    //     let mut input = String::new();
    //     io::stdin().read_line(&mut input).expect("Unable to read from stdin");
    //     input
    // }).trim();
    let input = {
        println!("Specify some input: ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Unable to read from stdin");
        input
    };
    let input = input.trim();

    let transmuted = {
        match option {
            "lowercase" => input.to_lowercase(),
            "uppercase" => input.to_uppercase(),
            "slugify" => slugify(input),
            "no-spaces" => input.replace(" ", ""),
            "len" => input.len().to_string(),
            "reverse" => input.chars().rev().collect::<String>(),
            _ => panic!("Unknown option")
        }
    };

    println!("{} => {}", input, transmuted);
}

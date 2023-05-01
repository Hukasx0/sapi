use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs::File, env, process};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    target: String,
    port: u16,
    endpoint: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    data: Option<HashMap<String, String>>,
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!("Usage: {} file.yml", argv[0]);
        process::exit(0);
    }
    let file = match File::open(&argv[1]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open {} file because of {}", argv[1], e);
            process::exit(1);
        }
    };
    let request_contents: Vec<Request> = match serde_yaml::from_reader(file) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("Error while parsing {} file: {}", argv[1], e);
            process::exit(1);
        }
    };
    for request in request_contents {
        println!("Found: {:?}", request);
    }
}

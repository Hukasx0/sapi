use serde::{Serialize, Deserialize};
use serde_json::to_writer_pretty;
use std::{collections::HashMap, fs::File, io::Write, time::Instant, env, process};
use ureq::Error;

#[derive(Serialize, Deserialize)]
struct Request {
    target: String,
    port: u16,
    endpoint: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    data: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    status: u16,
    status_text: String,
    method: String,
    url: String,
    server_response_time_ms: u128,
    response_body: String,
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() != 2 {
        println!("Usage: {} file.yml - to make HTTP requests from a file\n       {} new      - to generate a new skeleton file for making HTTP requests", argv[0], argv[0]);
        process::exit(0);
    } else if argv[1] == "new" {
        let example_yaml = r#"- target: <target>
  port: <port>
  endpoint: <path>
  method: <method>
#  headers:                         
  #  Authorization: Bearer <token>
  #  Content-Type: application/x-www-form-urlencoded
  #  Content-Type: application/json
  #  Content-Type: text/plain
  #  <header_name>: <header_value>
#  data:
  #  <value_name>: <value>
        "#;
        let mut file = match File::create("sapi.yml") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error while creating sapi.yml file: {}", e);
                process::exit(1);
            }
        };
        match file.write_all(example_yaml.as_bytes()) {
            Ok(_) => println!("sapi.yml created successfully!"),
            Err(e) => {
                eprintln!("Error while writing to sapi.yml file: {}", e);
                process::exit(1);
            }
        }
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
    let mut responses: Vec<Response> = Vec::new();
    for request in request_contents {
        match request.method.as_str() {
           "GET" | "HEAD" | "DELETE" => {
                let url = format!("http://{}:{}{}", request.target, request.port, request.endpoint);
                let mut req;
                match request.method.as_str() {
                    "GET" => req = ureq::get(&url),
                    "HEAD" => req = ureq::head(&url),
                    "DELETE" => req = ureq::delete(&url),
                    _ => {
                        eprintln!("Unexpected error");
                        process::exit(1);
                    }
                }
                if let Some(headers) = request.headers {
                    for (header, data) in headers {
                        req = req.set(&header, &data);
                    }
                }
                let req_start_time = Instant::now();
                let response = match req.call() {
                    Ok(r) => r,
                    Err(Error::Status(_, r)) => r,
                    Err(e) => {
                        eprintln!("Error while connecting to {}:{}: {}", request.target, request.port, e);
                        process::exit(1);
                    }
                };
                let req_end_time = Instant::now();
                println!("{:?}", response);
                responses.push(Response {
                    status: response.status(),
                    status_text: response.status_text().to_string(),
                    method: request.method,
                    url: url.to_string(),
                    server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                    response_body: match response.into_string() {
                        Ok(s) => s,
                        Err(_) => String::from(""),
                    },
                })
           },
           "POST" | "PUT" | "PATCH" => {
                let url = format!("http://{}:{}{}", request.target, request.port, request.endpoint);
                let mut req;
                match request.method.as_str() {
                    "POST" => req = ureq::post(&url),
                    "PUT" => req = ureq::put(&url),
                    "PATCH" => req = ureq::patch(&url),
                    _ => {
                        eprintln!("Unexpected error");
                        process::exit(1);
                    }
                }
                if let Some(headers) = request.headers {
                    for (header, data) in headers {
                        req = req.set(&header, &data);
                    }
                }
                if let Some(data) = request.data {
                    match req.header("Content-Type") {
                        Some(content_type) if content_type.starts_with("application/x-www-form-urlencoded") => {
                            let form = &data.iter().map(|(x, y)| (x.as_str(), y.as_str())).collect::<Vec<_>>();
                            let req_start_time = Instant::now();
                            match req.send_form(form) {
                                Ok(r) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: request.method,
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                },
                            }
                        },
                        Some(content_type) if content_type.starts_with("application/json") => {
                            let json_data = &serde_json::to_string(&serde_json::to_value(data).unwrap()).unwrap();
                            let req_start_time = Instant::now();
                            match req.send_string(json_data) {
                                Ok(r) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                },
                            }
                        },
                        Some(content_type) if content_type.starts_with("text/plain") => {
                            let plain_text = &data.get("txt").map(|v| v.to_string()).unwrap();
                            let req_start_time = Instant::now();
                            match req.send_string(plain_text) {
                                Ok(r) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    let req_end_time = Instant::now();
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                        server_response_time_ms: (req_end_time - req_start_time).as_millis(),
                                        response_body: match r.into_string() {
                                            Ok(s) => s,
                                            Err(_) => String::from(""),
                                        },
                                    });
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                },
                            }
                        },
                        Some(_) | None => {}
                    }
                }
           },
           _ => println!("Not supported request type") ,
        }
    }
    let file_json = match File::create("sapi.json") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error while creating sapi.json file: {}", e);
            process::exit(1);
        }
    };
    match to_writer_pretty(file_json, &responses) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error while trying to save results to sapi.json file: {}", e);
            process::exit(1);
        }
    }
    println!("Saved full requests data to sapi.json file");
}

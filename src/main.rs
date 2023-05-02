use serde::{Serialize, Deserialize};
use serde_json::to_writer_pretty;
use std::{collections::HashMap, fs::File, env, process};
use ureq::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    target: String,
    port: u16,
    endpoint: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    data: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    status: u16,
    status_text: String,
    method: String,
    url: String,
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
    let mut responses: Vec<Response> = Vec::new();
    for request in request_contents {
        match request.method.as_str() {
           "GET" => {
                let url = format!("http://{}:{}{}", request.target, request.port, request.endpoint);
                let response = match ureq::get(&url).call() {
                    Ok(r) => r,
                    Err(Error::Status(_, r)) => r,
                    Err(e) => {
                        eprintln!("Error while connecting to {}:{}: {}", request.target, request.port, e);
                        process::exit(1);
                    }
                };
                println!("{:?}", response);
                responses.push(Response {
                    status: response.status(),
                    status_text: response.status_text().to_string(),
                    method: String::from("GET"),
                    url: url.to_string(),
                })
           },
           "POST" => {
                let url = format!("http://{}:{}{}", request.target, request.port, request.endpoint);
                let mut req = ureq::post(&url);
                if let Some(headers) = request.headers {
                    for (header, data) in headers {
                        req = req.set(&header, &data);
                    }
                }
                if let Some(data) = request.data {
                    match req.header("Content-Type") {
                        Some(content_type) if content_type.starts_with("application/x-www-form-urlencoded") => {
                            match req.send_form(&data.iter().map(|(x, y)| (x.as_str(), y.as_str())).collect::<Vec<_>>()) {
                                Ok(r) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                    });
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                },
                            }
                        },
                        Some(content_type) if content_type.starts_with("application/json") => {
                            match req.send_string(&serde_json::to_string(&serde_json::to_value(data).unwrap()).unwrap()) {
                                Ok(r) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                    });
                                },
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                },
                            }
                        },
                        Some(content_type) if content_type.starts_with("text/plain") => {
                            match req.send_string(&data.get("txt").map(|v| v.to_string()).unwrap()) {
                                Ok(r) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
                                    });
                                },
                                Err(Error::Status(_, r)) => {
                                    println!("{:?}", r);
                                    responses.push(Response {
                                        status: r.status(),
                                        status_text: r.status_text().to_string(),
                                        method: String::from("POST"),
                                        url: url.to_string(),
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
    println!("{:?}", responses);
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
}

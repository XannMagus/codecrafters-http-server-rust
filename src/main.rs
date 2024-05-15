use std::env;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::http::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpResponseBuilder, MimeType};

mod http;

fn main() {
    // Get the directory flag if specified
    let mut directory = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--directory" => {
                directory = args.next();
            }
            _ => {}
        }
    }

    let directory = directory.unwrap_or(".".to_string());
    println!("Directory: {directory}\n");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response = handle_request(&stream, &directory);

                let response = match response {
                    Ok(good_response) => format!("{good_response}"),
                    Err(bad_response) => format!("{}", bad_response.to_response())
                };

                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_request(stream: &TcpStream, root: &String) -> Result<HttpResponse, HttpError> {
    let request = HttpRequest::from_stream(&stream)?;
    println!("{request}");

    if request.method != HttpMethod::GET {
        return Err(HttpError::MethodNotAllowed(vec!(HttpMethod::GET)));
    }

    match request.path.as_str() {
        "/" => Ok(HttpResponseBuilder::new().to_response()),
        "/user-agent" => {
            println!("Reading User-Agent Header");
            let mut response_builder = HttpResponseBuilder::new();
            if let Some(user_agent) = request.headers.get_value(&"User-Agent".to_string()) {
                response_builder = response_builder.with_body(user_agent, MimeType::PlainText);
            };
            Ok(response_builder.to_response())
        }
        path if path.starts_with("/echo/") => {
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let param = parts.get(1).unwrap_or(&"");
            println!("Echoing back the parameter {param}");
            let response = HttpResponseBuilder::new()
                .with_body(param.to_string(), MimeType::PlainText)
                .to_response();
            Ok(response)
        }
        path if path.starts_with("/files/") => {
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let param = parts.get(1).unwrap_or(&"");
            println!("Returning back the file {root}{param}");
            let body = get_file(root, &param.to_string())?;
            Ok(HttpResponseBuilder::new().with_body(body, MimeType::OctetStream).to_response())
        }
        path => Err(HttpError::NotFound(path.to_owned()))
    }
}

fn get_file(directory: &String, filename: &String) -> Result<String, HttpError> {
    let path = format!("{directory}{filename}");
    
    match File::open(path) {
        Ok(mut file) => {
            let mut buffer = String::new();
            match file.read_to_string(&mut buffer) {
                Ok(_) => {
                    Ok(buffer.clone())
                }
                Err(_) => {
                    Err(HttpError::InternalError)
                }
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Err(HttpError::NotFound(filename.clone()))
            } else {
                Err(HttpError::InternalError)
            }
        }
    }
}
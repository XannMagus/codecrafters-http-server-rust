use std::io::Write;
use std::net::{TcpListener, TcpStream};

use crate::http::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpResponseBuilder};

mod http;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let response = handle_request(&stream);

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

fn handle_request(stream: &TcpStream) -> Result<HttpResponse, HttpError> {
    let request = HttpRequest::from_stream(&stream)?;

    if request.method != HttpMethod::GET {
        return Err(HttpError::MethodNotAllowed(vec!(HttpMethod::GET)));
    }

    match request.path.as_str() {
        "/" => Ok(HttpResponseBuilder::new().to_response()),
        path if path.starts_with("/echo/") => {
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            let param = parts.get(1).unwrap_or(&"");
            println!("{path}: {parts:?}\nparam: {param}");
            let response = HttpResponseBuilder::new()
                .add_header("Content-Type".to_string(), "text/plain".to_string())
                .add_header(
                    "Content-Length".to_string(),
                    param.len().to_string(),
                )
                .with_body(param.to_string())
                .to_response();
            Ok(response)
        }
        path => Err(HttpError::NotFound(path.to_owned()))
    }
}
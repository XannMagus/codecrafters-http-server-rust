use std::io::Write;
use std::net::{TcpListener, TcpStream};

use crate::http::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpStatus, HttpVersion};

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
        return Err(HttpError::MethodNotAllowed(request.method));
    }

    match request.path.as_str() {
        "/" => Ok(HttpResponse::new(HttpVersion::V11, HttpStatus::OK)),
        path => Err(HttpError::NotFound(path.to_owned()))
    }
}
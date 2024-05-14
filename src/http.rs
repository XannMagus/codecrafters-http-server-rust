use std::fmt::{Display, Formatter};
use std::io::{BufReader, BufRead};
use std::net::TcpStream;

pub enum HttpStatus {
    OK,
    MethodNotAllowed,
    NotFound,
    BadRequest,
}

pub enum HttpVersion {
    V10,
    V11,
    V20,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    HEAD,
    CONNECT,
    PATCH,
    TRACE,
}

pub enum HttpError {
    NotFound(String),
    MethodNotAllowed(HttpMethod),
    BadRequest(ParseError),
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,
}

pub struct HttpResponse {
    version: HttpVersion,
    status: HttpStatus,
}

#[derive(Debug)]
pub enum ParseError {
    UnknownMethod(String),
    UnhandledVersion(String),
    MissingMethod,
    MissingVersion,
    MissingPath,
    MalformedRequest,
}

impl HttpRequest {
    pub fn from_stream(mut stream: &TcpStream) -> Result<Self, ParseError> {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_string = String::new();
        buf_reader.read_line(&mut request_string).or(Err(ParseError::MalformedRequest))?;
        let mut request_parts = request_string.split_ascii_whitespace();

        let Some(method) = request_parts.next() else {
            return Err(ParseError::MissingMethod);
        };
        let method = HttpMethod::from_str(method)?;

        let Some(path) = request_parts.next() else {
            return Err(ParseError::MissingPath);
        };

        let Some(version) = request_parts.next() else {
            return Err(ParseError::MissingVersion);
        };
        let version = HttpVersion::from_str(version)?;

        Ok(Self { method, path: path.to_string(), version })
    }
}

impl HttpMethod {
    pub fn from_str(input: &str) -> Result<Self, ParseError> {
        match input {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "HEAD" => Ok(HttpMethod::HEAD),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "TRACE" => Ok(HttpMethod::TRACE),
            _ => Err(ParseError::UnknownMethod(String::from(input)))
        }
    }
}

impl HttpVersion {
    pub fn from_str(input: &str) -> Result<Self, ParseError> {
        match input {
            "HTTP/1.0" => Ok(HttpVersion::V10),
            "HTTP/1.1" => Ok(HttpVersion::V11),
            "HTTP/2.0" => Ok(HttpVersion::V20),
            _ => Err(ParseError::UnhandledVersion(String::from(input)))
        }
    }
}

impl HttpResponse {
    pub fn new(version: HttpVersion, status: HttpStatus) -> Self {
        Self { version, status }
    }
}

impl HttpError {
    pub fn to_response(&self) -> HttpResponse {
        match self {
            HttpError::NotFound(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::NotFound),
            HttpError::MethodNotAllowed(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::MethodNotAllowed),
            HttpError::BadRequest(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::BadRequest),
        }
    }
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (code, description) = match self {
            HttpStatus::OK => (200, "OK"),
            HttpStatus::MethodNotAllowed => (405, "Method Not Allowed"),
            HttpStatus::NotFound => (404, "Not Found"),
            HttpStatus::BadRequest => (400, "Bad Request"),
        };

        write!(f, "{code} {description}")
    }
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string_version = match self {
            HttpVersion::V10 => "HTTP/1.0",
            HttpVersion::V11 => "HTTP/1.1",
            HttpVersion::V20 => "HTTP/2.0",
        };

        write!(f, "{string_version}")
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\r\n\r\n", self.version, self.status)
    }
}

impl From<ParseError> for HttpError {
    fn from(value: ParseError) -> Self {
        Self::BadRequest(value)
    }
}
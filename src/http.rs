use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::TcpStream;

use crate::http::parser::Parser;

mod parser;

pub enum HttpStatus {
    OK,
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalError,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum MimeType {
    PlainText,
    JSON,
    HTML,
    OctetStream,
}

pub enum HttpError {
    NotFound(String),
    MethodNotAllowed(Vec<HttpMethod>),
    BadRequest(ParseError),
    InternalError,
    Unauthorized,
    Forbidden,
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd,Copy, Clone)]
pub enum HttpEncoding {
    Gzip,
    Deflate,
    Brotli,
    Compress,
    Exi,
    Identity,
    Zstd,
    Unsupported,
}

#[derive(Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct HttpHeaderCollection {
    headers: HashMap<String, HttpHeader>,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,
    pub headers: HttpHeaderCollection,
    pub encoding: Option<HttpEncoding>,
    pub body: Option<Vec<u8>>,
}

pub struct HttpResponse {
    version: HttpVersion,
    status: HttpStatus,
    headers: HttpHeaderCollection,
    body: String,
}

pub struct HttpResponseBuilder {
    version: Option<HttpVersion>,
    status: Option<HttpStatus>,
    headers: HttpHeaderCollection,
    body: Option<String>,
}

#[derive(Debug)]
pub enum ParseError {
    UnknownMethod(String),
    UnhandledVersion(String),
    MissingMethod,
    MissingVersion,
    MissingPath,
    MalformedRequest,
    Unreachable,
}

impl HttpRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self, ParseError> {
        let parser = Parser::new();

        Ok(parser
            .parse(stream)?
            .get_request()
            .map_err(|_| ParseError::Unreachable)?)
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
            _ => Err(ParseError::UnknownMethod(String::from(input))),
        }
    }
}

impl HttpVersion {
    pub fn from_str(input: &str) -> Result<Self, ParseError> {
        match input {
            "HTTP/1.0" => Ok(HttpVersion::V10),
            "HTTP/1.1" => Ok(HttpVersion::V11),
            "HTTP/2.0" => Ok(HttpVersion::V20),
            _ => Err(ParseError::UnhandledVersion(String::from(input))),
        }
    }
}

impl HttpResponse {
    pub fn new(version: HttpVersion, status: HttpStatus) -> Self {
        Self {
            version,
            status,
            body: String::new(),
            headers: HttpHeaderCollection::new(),
        }
    }

    pub fn with_headers(
        version: HttpVersion,
        status: HttpStatus,
        headers: Vec<HttpHeader>,
    ) -> Self {
        let headers = HttpHeaderCollection::from_vector(headers);
        Self {
            version,
            status,
            headers,
            body: String::new(),
        }
    }
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            version: None,
            status: None,
            headers: HttpHeaderCollection::new(),
            body: None,
        }
    }
    /*
        pub fn with_version(mut self, version: HttpVersion) -> Self {
            self.version = Some(version);
            self
        }
    */
    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_body(
        mut self,
        body: String,
        mime_type: MimeType,
        encoding: Option<HttpEncoding>,
    ) -> Self {
        self.headers
            .add_header("Content-Type".to_string(), mime_type.to_string());
        self.headers
            .add_header("Content-Length".to_string(), body.len().to_string());
        if let Some(encoding) = encoding {
            if encoding != HttpEncoding::Unsupported {
                self.headers
                    .add_header("Content-Encoding".to_string(), encoding.to_string());
            }
        }
        self.body = Some(body);
        self
    }

    pub fn add_header(mut self, name: String, value: String) -> Self {
        self.headers.add_header(name, value);
        self
    }

    /*
        pub fn add_header_object(mut self, header: HttpHeader) -> Self {
            self.headers.add_header_object(header);
            self
        }

        pub fn add_headers(mut self, headers: Vec<HttpHeader>) -> Self {
            self.headers.add_vector(headers);
            self
        }
    */
    pub fn to_response(self) -> HttpResponse {
        let version = self.version.unwrap_or(HttpVersion::V11);
        let status = self.status.unwrap_or(HttpStatus::OK);
        let body = self.body.unwrap_or(String::new());

        HttpResponse {
            version,
            status,
            headers: self.headers,
            body,
        }
    }
}

impl HttpError {
    pub fn to_response(&self) -> HttpResponse {
        match self {
            HttpError::NotFound(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::NotFound),
            HttpError::MethodNotAllowed(methods) => {
                let allowed_methods = methods
                    .iter()
                    .map(|method| method.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                println!("{allowed_methods}");
                HttpResponse::with_headers(
                    HttpVersion::V11,
                    HttpStatus::MethodNotAllowed,
                    vec![HttpHeader::new(String::from("Allowed"), allowed_methods)],
                )
            }
            HttpError::BadRequest(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::BadRequest),
            HttpError::InternalError => HttpResponseBuilder::new()
                .with_status(HttpStatus::InternalError)
                .to_response(),
            HttpError::Unauthorized => HttpResponseBuilder::new()
                .with_status(HttpStatus::Unauthorized)
                .add_header("WWW-Authenticate".to_string(), "Basic".to_string())
                .to_response(),
            HttpError::Forbidden => HttpResponseBuilder::new()
                .with_status(HttpStatus::Forbidden)
                .to_response(),
        }
    }
}

impl From<HttpError> for HttpResponse {
    fn from(value: HttpError) -> Self {
        value.to_response()
    }
}

impl HttpHeader {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl HttpHeaderCollection {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn from_vector(input: Vec<HttpHeader>) -> Self {
        let mut headers = HashMap::new();

        for header in input.into_iter() {
            headers.insert(header.name.clone(), header);
        }

        Self { headers }
    }

    /*
        pub fn get(&self, key: &String) -> Option<&HttpHeader> {
            self.headers.get(key)
        }
    */
    pub fn get_value(&self, key: &String) -> Option<String> {
        if let Some(header) = self.headers.get(key) {
            Some(header.value.clone())
        } else {
            None
        }
    }

    pub fn add_header(&mut self, key: String, value: String) {
        let header = HttpHeader::new(key.clone(), value);
        self.headers.insert(key, header);
    }
    /*
    pub fn add_header_object(&mut self, header: HttpHeader) {
        self.headers.insert(header.name.clone(), header);
    }

    pub fn add_vector(&mut self, headers: Vec<HttpHeader>) {
        for header in headers {
            self.add_header_object(header);
        }
    }
    */
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (code, description) = match self {
            HttpStatus::OK => (200, "OK"),
            HttpStatus::MethodNotAllowed => (405, "Method Not Allowed"),
            HttpStatus::NotFound => (404, "Not Found"),
            HttpStatus::BadRequest => (400, "Bad Request"),
            HttpStatus::InternalError => (500, "Internal Server Error"),
            HttpStatus::Created => (201, "Created"),
            HttpStatus::Unauthorized => (401, "Unauthorized"),
            HttpStatus::Forbidden => (403, "Forbidden"),
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
        write!(
            f,
            "{} {}\r\n{}\r\n{}",
            self.version, self.status, self.headers, self.body
        )
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let body_string = match &self.body {
            None => "",
            Some(content) => std::str::from_utf8(content).map_err(|_| std::fmt::Error)?,
        };
        write!(
            f,
            "{} {} {}\r\n{}\r\n{body_string}",
            self.method, self.path, self.version, self.headers
        )
    }
}

impl Display for HttpHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}\r\n", self.name, self.value)
    }
}

impl Display for HttpHeaderCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = std::fmt::Result::Ok(());
        for header in self.headers.values() {
            result = write!(f, "{}", header);
        }
        result
    }
}

impl Display for MimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string_representation = match self {
            MimeType::PlainText => "text/plain",
            MimeType::JSON => "application/json",
            MimeType::HTML => "text/html",
            MimeType::OctetStream => "application/octet-stream",
        };
        write!(f, "{string_representation}")
    }
}

impl From<ParseError> for HttpError {
    fn from(value: ParseError) -> Self {
        Self::BadRequest(value)
    }
}

impl From<&str> for HttpEncoding {
    fn from(value: &str) -> Self {
        match value {
            "br" => Self::Brotli,
            "gzip" => Self::Gzip,
            "deflate" => Self::Deflate,
            "compress" => Self::Compress,
            "exi" => Self::Exi,
            "zstd" => Self::Zstd,
            "identity" => Self::Identity,
            &_ => Self::Unsupported,
        }
    }
}

impl From<String> for HttpEncoding {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for HttpEncoding {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for HttpEncoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string_representation = match self {
            HttpEncoding::Brotli => "br",
            HttpEncoding::Compress => "compress",
            HttpEncoding::Deflate => "deflate",
            HttpEncoding::Exi => "exi",
            HttpEncoding::Gzip => "gzip",
            HttpEncoding::Identity => "identity",
            HttpEncoding::Zstd => "zstd",
            HttpEncoding::Unsupported => "",
        };
        write!(f, "{string_representation}")
    }
}

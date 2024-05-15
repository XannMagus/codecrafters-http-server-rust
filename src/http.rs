use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
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
    MethodNotAllowed(Vec<HttpMethod>),
    BadRequest(ParseError),
}

pub struct HttpHeader {
    name: String,
    value: String,
}

pub struct HttpHeaderCollection {
    headers: HashMap<String, HttpHeader>,
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: HttpVersion,
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
        Self { version, status, body: String::new(), headers: HttpHeaderCollection::new() }
    }

    pub fn with_headers(version: HttpVersion, status: HttpStatus, headers: Vec<HttpHeader>) -> Self {
        let headers = HttpHeaderCollection::from_vector(headers);
        Self { version, status, headers, body: String::new() }
    }
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self { version: None, status: None, headers: HttpHeaderCollection::new(), body: None }
    }
/*
    pub fn with_version(mut self, version: HttpVersion) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = Some(status);
        self
    }
*/
    pub fn with_body(mut self, body: String) -> Self {
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
            body
        }
    }
}


impl HttpError {
    pub fn to_response(&self) -> HttpResponse {
        match self {
            HttpError::NotFound(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::NotFound),
            HttpError::MethodNotAllowed(methods) => {
                let allowed_methods = methods.iter().map(|method| method.to_string()).collect::<Vec<String>>().join(",");
                println!("{allowed_methods}");
                HttpResponse::with_headers(HttpVersion::V11, HttpStatus::MethodNotAllowed, vec!(HttpHeader::new(String::from("Allowed"), allowed_methods)))
            }
            HttpError::BadRequest(_) => HttpResponse::new(HttpVersion::V11, HttpStatus::BadRequest),
        }
    }
}

impl HttpHeader {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl HttpHeaderCollection {
    pub fn new() -> Self {
        Self { headers: HashMap::new() }
    }

    pub fn from_vector(input: Vec<HttpHeader>) -> Self {
        let mut headers = HashMap::new();

        for header in input.into_iter() {
            headers.insert(header.name.clone(), header);
        }

        Self { headers }
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
        write!(f, "{} {}\r\n{}\r\n{}", self.version, self.status, self.headers, self.body)
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

impl From<ParseError> for HttpError {
    fn from(value: ParseError) -> Self {
        Self::BadRequest(value)
    }
}
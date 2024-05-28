use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read};

use crate::http::{
    HttpEncoding, HttpHeaderCollection, HttpMethod, HttpRequest, HttpVersion,
    ParseError,
};

#[derive(Eq, PartialEq)]
enum ParserState {
    Start,
    Headers,
    Body,
    Done,
}

pub struct Parser {
    state: ParserState,
    method: Option<HttpMethod>,
    path: Option<String>,
    version: Option<HttpVersion>,
    headers: HttpHeaderCollection,
    body: Option<Vec<u8>>,
    content_length: Option<usize>,
    content_encoding: BTreeSet<HttpEncoding>,
}

pub struct Unparsed;

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Start,
            method: None,
            path: None,
            version: None,
            headers: HttpHeaderCollection::new(),
            body: None,
            content_length: None,
            content_encoding: BTreeSet::new(),
        }
    }

    pub fn get_request(self) -> Result<HttpRequest, Unparsed> {
        if self.state != ParserState::Done {
            return Err(Unparsed);
        }

        Ok(HttpRequest {
            method: self.method.unwrap(),
            path: self.path.unwrap(),
            version: self.version.unwrap(),
            headers: self.headers,
            encoding: self.content_encoding.iter().next().copied(),
            body: self.body,
        })
    }

    pub fn parse<R>(mut self, stream: R) -> Result<Self, ParseError>
    where
        R: Read,
    {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        while self.state != ParserState::Done && self.state != ParserState::Body {
            line.clear();
            let bytes_read = reader
                .read_line(&mut line)
                .map_err(|_| ParseError::MalformedRequest)?;
            if bytes_read == 0 {
                break;
            }
            match self.state {
                ParserState::Start => self.parse_start_line(&line)?,
                ParserState::Headers => self.parse_header(&line)?,
                ParserState::Body => Err(ParseError::Unreachable)?,
                ParserState::Done => Err(ParseError::Unreachable)?,
            };
        }

        if self.state == ParserState::Body {
            self.parse_body(&mut reader)?;
        }
        Ok(self)
    }

    fn parse_start_line(&mut self, line: &String) -> Result<(), ParseError> {
        let mut parts = line.split_ascii_whitespace();

        let Some(method) = parts.next() else {
            return Err(ParseError::MissingMethod);
        };
        self.method = Some(HttpMethod::from_str(method)?);

        let Some(path) = parts.next() else {
            return Err(ParseError::MissingPath);
        };
        self.path = Some(path.to_string());

        let Some(version) = parts.next() else {
            return Err(ParseError::MissingVersion);
        };
        self.version = Some(HttpVersion::from_str(version)?);

        self.state = ParserState::Headers;
        Ok(())
    }

    fn parse_header(&mut self, line: &String) -> Result<(), ParseError> {
        if line.trim().is_empty() {
            self.state = if self.content_length.is_some() {
                ParserState::Body
            } else {
                ParserState::Done
            };
            return Ok(());
        }

        let parts: Vec<&str> = line.split(": ").collect();
        if parts.len() < 2 {
            return Err(ParseError::MalformedRequest);
        }
        let (key, value) = (parts[0].trim().to_string(), parts[1].trim().to_string());
        if key.eq_ignore_ascii_case("Content-Length") {
            self.content_length = Some(value.parse().map_err(|_| ParseError::MalformedRequest)?);
        }
        if key.eq_ignore_ascii_case("Accept-Encoding") {
            for encoding_string in value.split(",") {
                self.content_encoding.insert(HttpEncoding::from(encoding_string.trim()));
            }
        }

        self.headers.add_header(key, value);
        Ok(())
    }

    fn parse_body<R>(&mut self, reader: &mut BufReader<R>) -> Result<(), ParseError>
    where
        R: Read,
    {
        if let Some(length) = self.content_length {
            let mut buffer = vec![0; length];
            reader
                .read_exact(&mut buffer)
                .map_err(|_| ParseError::MalformedRequest)?;
            self.body = Some(buffer);
        }

        self.state = ParserState::Done;
        Ok(())
    }
}

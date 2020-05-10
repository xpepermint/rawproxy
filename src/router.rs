use async_std::prelude::*;
use crate::{Stream, SocketAddr, Error};
use crate::utils::{read_protocol, read_header, write_header, remove_header, forward_body};

#[derive(Debug)]
pub struct Router {
    stream: Stream,
    relay: Option<Stream>,
    request_headers: Vec<String>,
    response_headers: Vec<String>,
    request_headers_size_limit: Option<usize>,
    request_body_size_limit: Option<usize>,
    response_headers_size_limit: Option<usize>,
    response_body_size_limit: Option<usize>,
}

impl Router {

    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            relay: None,
            request_headers: Vec::new(),
            response_headers: Vec::new(),
            request_headers_size_limit: None,
            request_body_size_limit: None,
            response_headers_size_limit: None,
            response_body_size_limit: None,
        }
    }

    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    pub fn request_headers(&self) -> &Vec<String> {
        &self.request_headers
    }

    pub fn response_headers(&self) -> &Vec<String> {
        &self.request_headers
    }

    pub fn request_header<N: Into<String>>(&self, name: N) -> Option<String> {
        read_header(&self.request_headers, &name.into())
    }

    pub fn response_header<N: Into<String>>(&self, name: N) -> Option<String> {
        read_header(&self.response_headers, &name.into())
    }

    pub fn request_headers_size_limit(&self) -> &Option<usize> {
        &self.request_headers_size_limit
    }

    pub fn request_body_size_limit(&self) -> &Option<usize> {
        &self.request_body_size_limit
    }

    pub fn response_headers_size_limit(&self) -> &Option<usize> {
        &self.response_headers_size_limit
    }

    pub fn response_body_size_limit(&self) -> &Option<usize> {
        &self.response_body_size_limit
    }

    pub fn set_request_header<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) {
        write_header(&mut self.request_headers, &name.into(), &value.into());
    }

    pub fn set_response_header<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) {
        write_header(&mut self.response_headers, &name.into(), &value.into());
    }

    pub fn set_request_headers_size_limit(&mut self, limit: usize) {
        self.request_headers_size_limit = Some(limit);
    }

    pub fn set_request_body_size_limit(&mut self, limit: usize) {
        self.request_body_size_limit = Some(limit);
    }

    pub fn set_response_headers_size_limit(&mut self, limit: usize) {
        self.response_headers_size_limit = Some(limit);
    }

    pub fn set_response_body_size_limit(&mut self, limit: usize) {
        self.response_body_size_limit = Some(limit);
    }

    pub fn remove_request_header<N: Into<String>>(&mut self, name: N) {
        remove_header(&mut self.request_headers, &name.into());
    }

    pub fn remove_response_header<N: Into<String>>(&mut self, name: N) {
        remove_header(&mut self.response_headers, &name.into());
    }

    pub fn remove_request_headers_size_limit(&mut self) {
        self.request_headers_size_limit = None;
    }

    pub fn remove_request_body_size_limit(&mut self) {
        self.request_body_size_limit = None;
    }

    pub fn remove_response_headers_size_limit(&mut self) {
        self.response_headers_size_limit = None;
    }

    pub fn remove_response_body_size_limit(&mut self) {
        self.response_body_size_limit = None;
    }

    pub async fn parse_request(&mut self) -> Result<(), Error> {
        self.request_headers.clear();
        match read_protocol(&mut self.stream, &mut self.request_headers, self.request_headers_size_limit).await {
            Ok(_) => Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn parse_response(&mut self) -> Result<(), Error> {
        let mut source = match &mut self.relay {
            Some(source) => source,
            None => return Err(Error::StreamNotReadable),
        };
        self.response_headers.clear();
        match read_protocol(&mut source, &mut self.response_headers, self.response_headers_size_limit).await {
            Ok(_) => Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn relay_request(&mut self) -> Result<(), Error> {
        let host = match read_header(&self.request_headers, "Host") {
            Some(host) => host,
            None => return Err(Error::MissingHeader(String::from("Host"))),
        };
        let address = match SocketAddr::from_str(host).await {
            Ok(address) => address,
            Err(_) => return Err(Error::InvalidHeader(String::from("Host"))),
        };
        self.relay = match Stream::connect(&address).await {
            Ok(source) => Some(source),
            Err(_) => return Err(Error::StreamNotWritable),
        };
        let mut target = &mut self.relay.as_ref().unwrap();
        match target.write(self.request_headers.join("\r\n").as_bytes()).await {
            Ok(_) => (),
            Err(_) => return Err(Error::StreamNotWritable),
        };
        match forward_body(&mut self.stream, &mut target, &self.request_headers, self.request_body_size_limit).await {
            Ok(_) => Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn relay_response(&mut self) -> Result<(), Error> {
        let mut source = match &mut self.relay {
            Some(source) => source,
            None => return Err(Error::StreamNotWritable),
        };
        match self.stream.write(self.response_headers.join("\r\n").as_bytes()).await {
            Ok(_) => (),
            Err(_) => return Err(Error::StreamNotWritable),
        };
        match forward_body(&mut source, &mut self.stream, &self.response_headers, self.response_body_size_limit).await {
            Ok(_) => Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        match self.stream.write(bytes).await {
            Ok(size) => Ok(size),
            Err(_) => return Err(Error::StreamNotWritable),
        }
    }
}

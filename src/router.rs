use async_std::io;
use async_std::prelude::*;
use crate::{Stream, SocketAddr, Error};
use crate::utils::{read_protocol, read_header, write_header, forward_body};

#[derive(Debug)]
pub struct Router {
    stream: Stream,
    relay: Option<Stream>,
    request_headers: Vec<String>,
    response_headers: Vec<String>,
}

impl Router {

    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            relay: None,
            request_headers: Vec::new(),
            response_headers: Vec::new(),
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

    pub fn read_request_header<N: Into<String>>(&self, name: N) -> Option<String> {
        read_header(&self.request_headers, &name.into())
    }

    pub fn read_response_header<N: Into<String>>(&self, name: N) -> Option<String> {
        read_header(&self.response_headers, &name.into())
    }

    pub fn write_request_header<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) {
        write_header(&mut self.request_headers, &name.into(), &value.into());
    }

    pub fn write_response_header<N: Into<String>, V: Into<String>>(&mut self, name: N, value: V) {
        write_header(&mut self.response_headers, &name.into(), &value.into());
    }

    pub async fn parse_request(&mut self) -> Result<(), Error> {
        match read_protocol(&mut self.stream, &mut self.request_headers, None).await {
            Ok(_) => Ok(()),
            Err(error) => return Err(error),
        }
    }

    pub async fn parse_response(&mut self) -> Result<(), Error> {
        let mut source = match &mut self.relay {
            Some(source) => source,
            None => return Err(Error::StreamNotReadable),
        };
        match read_protocol(&mut source, &mut self.response_headers, None).await {
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
        match forward_body(&mut self.stream, &mut target, &self.request_headers, None).await {
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
        match forward_body(&mut source, &mut self.stream, &self.response_headers, None).await {
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

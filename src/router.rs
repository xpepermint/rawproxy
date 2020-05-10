use async_std::io;
use async_std::prelude::*;
use async_std::sync::{Arc};
use crate::{Stream, RouterOptions, SocketAddr, Error, ErrorKind};
use crate::utils::{read_protocol, read_header, write_header, forward_body};

pub struct Router {
    stream: Stream,
    options: Arc<RouterOptions>,
}

impl Router {
    
    pub fn from_stream(stream: Stream) -> Self {
        Self {
            stream,
            options: Arc::new(RouterOptions::default()),
        }
    }

    pub fn stream(&self) -> &Stream {
        &self.stream
    }

    pub fn options(&self) -> &RouterOptions {
        &self.options
    }

    pub fn set_stream(&mut self, stream: Stream) {
        self.stream = stream;
    }

    pub fn set_options(&mut self, options: Arc<RouterOptions>) {
        self.options = options;
    }

    pub async fn resolve(&mut self) -> io::Result<()> {
    
        // read request headers
        let mut lines: Vec<String> = Vec::new();
        match read_protocol(&mut self.stream, &mut lines, *self.options.request_headers_size_limit()).await {
            Ok(_) => (),
            Err(kind) => return self.abort(Error::Request(kind)).await,
        };

        // set target Host header
        let host = match read_header(&lines, "Host") {
            Some(host) => match self.options.target(&host) {
                Some(host) => host.to_string(),
                None => host,
            },
            None => return self.abort(Error::Request(ErrorKind::MissingHeader(String::from("Host")))).await,
        };
        write_header(&mut lines, "Host", &host);

        // prepare target address
        let address = match SocketAddr::from_str(host).await {
            Ok(address) => address,
            Err(_) => return self.abort(Error::Relay(ErrorKind::InvalidHeader(String::from("Host")))).await,
        };
    
        // read response
        // forward request to target
        let mut source = match Stream::connect(&address).await {
            Ok(source) => source,
            Err(_) => return self.abort(Error::Relay(ErrorKind::WriteFailed)).await,
        };
        match source.write(lines.join("\r\n").as_bytes()).await {
            Ok(_) => (),
            Err(_) => return self.abort(Error::Relay(ErrorKind::WriteFailed)).await,
        };
        match forward_body(&mut self.stream, &mut source, &lines, *self.options.request_body_size_limit()).await {
            Ok(_) => (),
            Err(kind) => return self.abort(Error::Relay(kind)).await,
        };

        // parse response protocol headers
        let mut lines: Vec<String> = Vec::new();
        match read_protocol(&mut source, &mut lines, *self.options.response_headers_size_limit()).await {
            Ok(_) => (),
            Err(kind) => return self.abort(Error::Response(kind)).await,
        };

        // forward response to client
        match self.stream.write(lines.join("\r\n").as_bytes()).await {
            Ok(_) => (),
            Err(_) => return self.abort(Error::Response(ErrorKind::WriteFailed)).await,
        };
        match forward_body(&mut source, &mut self.stream, &lines, *self.options.response_body_size_limit()).await {
            Ok(_) => (),
            Err(kind) => return self.abort(Error::Response(kind)).await,
        };

        Ok(())
    }

    async fn abort(&mut self, error: Error) -> io::Result<()> {
        println!("error: {:?}", error);
        self.stream.write(b"ERROR!").await.unwrap();
        Ok(())
    }
}

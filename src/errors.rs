#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Request(ErrorKind),
    Relay(ErrorKind),
    Response(ErrorKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    SizeLimitExceeded(usize),
    ReadFailed,
    WriteFailed,
    InvalidData,
    InvalidHeader(String),
    MissingHeader(String),
}

use std::{
    io::{self, Write},
    net::{AddrParseError, SocketAddr, TcpStream},
    result,
    str::FromStr,
};
use thiserror::Error;

pub type Result = result::Result<Connection, Error>;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    password: Option<String>,
}

impl Connection {
    const fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            password: None,
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseError(AddrParseError),
    #[error(transparent)]
    ConnectionError(io::Error),
}

// TODO: maybe make async some day when doing heavy computation
pub fn try_connect(uri: &str) -> Result {
    let address = SocketAddr::from_str(&uri)?;

    let mut stream = TcpStream::connect(address)?;

    stream.write_all(b"Hello from the rust program Overseer\n")?;

    Ok(Connection::new(stream))
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::ConnectionError(err)
    }
}

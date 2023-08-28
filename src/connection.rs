use async_std::net::TcpStream;
use iced::futures::AsyncWriteExt;
use std::{
    io,
    net::{AddrParseError, SocketAddr},
    result,
    str::FromStr,
    sync::Arc,
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
    ParseError(#[from] AddrParseError),
    #[error(transparent)]
    ConnectionError(#[from] io::Error),
}

pub async fn try_connect(uri: Arc<str>) -> Result {
    let address = SocketAddr::from_str(&uri)?;

    let mut stream = TcpStream::connect(address).await?;

    stream
        .write_all(b"Hello from the rust program Overseer\n")
        .await?;

    Ok(Connection::new(stream))
}

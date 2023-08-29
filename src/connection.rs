use async_std::{io::ReadExt, net::TcpStream};
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

const DEFAULT_XENA_PASSWORD: &str = "xena";

#[derive(Debug)]
pub enum Connection {
    ConnectionEstablished(ConnectionEstablished),
    LoggedIn(LoggedIn),
}

pub async fn connect(uri: Arc<str>) -> result::Result<ConnectionEstablished, Error> {
    let address = SocketAddr::from_str(&uri)?;

    let stream = TcpStream::connect(address).await?;

    Ok(ConnectionEstablished { stream })
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AddrParse(#[from] AddrParseError),
    #[error(transparent)]
    Connection(#[from] io::Error),
    #[error("Invalid Password")]
    InvalidPassword,
    #[error(transparent)]
    Parse(#[from] std::str::Utf8Error),
}

#[derive(Debug)]
pub struct ConnectionEstablished {
    stream: TcpStream,
}

#[derive(Debug)]
pub struct LoggedIn {
    stream: TcpStream,
}

impl From<ConnectionEstablished> for Connection {
    fn from(connection: ConnectionEstablished) -> Self {
        Self::ConnectionEstablished(connection)
    }
}

impl ConnectionEstablished {
    pub async fn log_in(mut self) -> result::Result<LoggedIn, Error> {
        self.stream
            .write_all(format!("C_LOGON \"{DEFAULT_XENA_PASSWORD}\"\n").as_bytes())
            .await?;

        let mut buf = [0u8; 32];
        let bytes_read = self.stream.read(&mut buf).await?;

        let response = std::str::from_utf8(&buf[..bytes_read])?;

        if response == "<OK>\n" {
            Ok(LoggedIn {
                stream: self.stream,
            })
        } else {
            Err(Error::InvalidPassword)
        }
    }
}

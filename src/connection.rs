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

const DEFAULT_XENA_PASSWORD: &str = "xena";

// represent an a logged in connection to a xena
#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
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

impl Connection {
    pub async fn connect(uri: Arc<str>) -> Result<Connection, Error> {
        let address = SocketAddr::from_str(&uri)?;

        let stream = TcpStream::connect(address).await?;

        let mut connection = Connection { stream };
        connection.log_in().await?;

        Ok(connection)
    }

    async fn log_in(&mut self) -> result::Result<(), Error> {
        self.stream
            .write_all(format!("C_LOGON \"{DEFAULT_XENA_PASSWORD}\"\n").as_bytes())
            .await?;

        let mut buf = [0u8; 32];
        let bytes_read = self.stream.read(&mut buf).await?;

        let response = std::str::from_utf8(&buf[..bytes_read])?;

        if response == "<OK>\n" {
            Ok(())
        } else {
            Err(Error::InvalidPassword)
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ignored = futures::executor::block_on(self.stream.write_all(b"C_LOGOFF\n"));
    }
}

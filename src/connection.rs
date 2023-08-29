use async_std::{future, io::ReadExt, net::TcpStream};
use iced::futures::AsyncWriteExt;
use std::{
    io,
    net::{AddrParseError, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
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

    async fn log_in(&mut self) -> Result<(), Error> {
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

    pub async fn list_interfaces(mut self) -> Result<(Self, Vec<String>), Error> {
        self.stream.write_all(b"*/* P_INTERFACE ?\n").await?;

        let mut buf = [0u8; 2048];

        let mut interfaces: Vec<String> = Vec::new();
        loop {
            if future::timeout(Duration::from_millis(100), self.stream.read(&mut buf))
                .await
                .is_err()
            {
                break;
            }

            let bytes_read = self.stream.read(&mut buf).await?;

            let stream_input = std::str::from_utf8(&buf[..bytes_read])?;

            interfaces.extend(
                stream_input
                    .split('\n')
                    .filter(|str| !str.is_empty())
                    .map(str::to_string),
            );
        }
        Ok((self, interfaces))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ignored = futures::executor::block_on(self.stream.write_all(b"C_LOGOFF\n"));
    }
}

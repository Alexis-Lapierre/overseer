use std::{
    io::{self, Read, Write},
    net::{AddrParseError, SocketAddr, TcpStream},
    ops::Deref,
    str::FromStr,
    sync,
};
use thiserror::Error;

mod interface;
pub use interface::*;

const DEFAULT_XENA_PASSWORD: &str = "xena";

// represent an a logged in connection to a xena
#[derive(Debug, Clone)]
pub struct Connection {
    stream: sync::mpsc::Sender<Command>,
}

type Responder<T> = oneshot::Sender<T>;
enum Command {
    ListInterfaces(Responder<Result<Interfaces, Error>>),
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
    Parse(#[from] anyhow::Error),
    #[error("TCP connection parse Error")]
    TCPParse,
}

impl From<interface::Error> for Error {
    fn from(value: interface::Error) -> Self {
        Self::Parse(value.into())
    }
}

impl Connection {
    pub fn connect(uri: impl Deref<Target = str>) -> Result<Connection, Error> {
        let address = SocketAddr::from_str(&uri)?;

        let stream = TcpStream::connect(address)?;

        let mut stream = log_in(stream)?;

        let (tx, rx) = sync::mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    Command::ListInterfaces(responce) => {
                        let _ = responce.send(list_interfaces(&mut stream));
                    }
                }
            }

            let _ = stream.write_all(b"C_LOGOFF\n");
        });

        Ok(Connection { stream: tx })
    }

    pub fn list_interfaces(self) -> Result<Interfaces, Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.stream
            .send(Command::ListInterfaces(resp_tx))
            .expect("Could not send to TCP handeling thread !");
        resp_rx
            .recv()
            .expect("Could not receive from TCP handeling thread !")
    }
}

fn log_in(mut stream: TcpStream) -> Result<TcpStream, Error> {
    stream
        .write_all(const_format::formatcp!("C_LOGON \"{DEFAULT_XENA_PASSWORD}\"\n").as_bytes())?;

    let mut buf = [0u8; 32];
    let bytes_read = stream.read(&mut buf)?;

    let response = std::str::from_utf8(&buf[..bytes_read])
        .expect("TCP connection received invalid UTF-8 character !");

    if response == "<OK>\n" {
        Ok(stream)
    } else {
        Err(Error::InvalidPassword)
    }
}

pub fn list_interfaces(stream: &mut TcpStream) -> Result<Interfaces, Error> {
    // We send two line feed, this way the end of the command is detected we encountering empty line ("\n\n")
    stream.write_all(b"*/* P_RESERVATION ?\n\n")?;

    let mut buf = [0u8; 2048];

    let mut interfaces = Interfaces::default();
    loop {
        let bytes_read = stream.read(&mut buf)?;

        let stream_input = std::str::from_utf8(&buf[..bytes_read])
            .expect("TCP connection received invalid UTF-8 characters !");

        let module_list = stream_input
            .split('\n')
            .filter(|str| !str.is_empty())
            .map(parse_interface_from_line);

        for maybe_elem in module_list {
            let (module, port, state) = maybe_elem?;
            interfaces
                .modules
                .entry(module)
                .or_default()
                .insert(port, state);
        }

        if stream_input.ends_with("\n\n") {
            return Ok(interfaces);
        }
    }
}

fn parse_interface_from_line<'a>(line: &'a str) -> Result<(u8, u8, State), Error> {
    debug_assert!(!line.is_empty());
    let get_module_and_port = |line: &'a str| -> Option<(&'a str, &'a str, &'a str)> {
        let mut line_per_space = line.split(' ');
        let mut module_and_port = line_per_space.next()?.split('/');
        let module = module_and_port.next()?;
        let port = module_and_port.next()?;

        let state = line_per_space.last()?;
        Some((module, port, state))
    };

    let (module, port, state) = get_module_and_port(line).ok_or(Error::TCPParse)?;

    let module = u8::from_str(module).map_err(|_| Error::TCPParse)?;
    let port = u8::from_str(port).map_err(|_| Error::TCPParse)?;
    let state = State {
        lock: state.try_into()?,
    };

    Ok((module, port, state))
}

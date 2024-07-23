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

macro_rules! formatcpln {
    ($format_string:expr $( $(, $expr:expr )+ )? $(,)? ) => {
        const_format::concatcp!(
            const_format::formatcp!(
                $format_string
                $(, $($expr,)+)?
            ), '\n'
        )
    };
}

// represent an a logged in connection to a xena
#[derive(Debug, Clone)]
pub struct Connection {
    stream: sync::mpsc::Sender<Command>,
}

type Responder<T> = oneshot::Sender<T>;
enum Command {
    ListInterfaces(Responder<Result<Interfaces, Error>>),
    LockInterface(u8, u8, Responder<Result<(), Error>>),
    UnlockInterface(u8, u8, Responder<Result<(), Error>>),
    RelinquishInterface(u8, u8, Responder<Result<(), Error>>),
}

impl Command {
    const fn from_lock_state(
        state: Lock,
        module: u8,
        port: u8,
        resp_tx: Responder<Result<(), Error>>,
    ) -> Self {
        match state {
            Lock::Released => Command::LockInterface(module, port, resp_tx),
            Lock::ReservedByYou => Command::UnlockInterface(module, port, resp_tx),
            Lock::ReservedByOther => Command::RelinquishInterface(module, port, resp_tx),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AddrParse(#[from] AddrParseError),
    #[error(transparent)]
    Connection(#[from] io::Error),
    #[error(r#"Responce is not "<OK>""#)]
    NotOk,
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

        let mut stream = Stream::new(stream)?;

        let (tx, rx) = sync::mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    Command::ListInterfaces(responce) => {
                        let _ = responce.send(stream.list_interfaces());
                    }

                    Command::LockInterface(module, port, responce) => {
                        let _ = responce.send(stream.lock_interface(module, port));
                    }

                    Command::UnlockInterface(module, port, responce) => {
                        let _ = responce.send(stream.unlock_interface(module, port));
                    }

                    Command::RelinquishInterface(module, port, responce) => {
                        let _ = responce.send(stream.relinquish_interface(module, port));
                    }
                }
            }

            let _ = stream.stream.write_all(b"C_LOGOFF\n");
        });

        Ok(Connection { stream: tx })
    }

    pub fn list_interfaces(self) -> Result<Interfaces, Error> {
        self.send_order(Command::ListInterfaces)
    }

    pub fn lock_action_on(&self, current_state: Lock, module: u8, port: u8) -> Result<(), Error> {
        self.send_order(|tx| Command::from_lock_state(current_state, module, port, tx))
    }

    fn send_order<F, T>(&self, command_builder: F) -> Result<T, Error>
    where
        F: FnOnce(Responder<Result<T, Error>>) -> Command,
    {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command = command_builder(resp_tx);
        self.stream
            .send(command)
            .expect("Could not send to TCP handeling thread !");
        resp_rx
            .recv()
            .expect("Could not receive from TCP handeling thread !")
    }
}

struct Stream {
    stream: TcpStream,
}

impl Stream {
    fn new(mut stream: TcpStream) -> Result<Self, Error> {
        stream.write_all(formatcpln!(r#"C_LOGON "{DEFAULT_XENA_PASSWORD}""#).as_bytes())?;

        let mut buf = [0u8; 32];
        let bytes_read = stream.read(&mut buf)?;

        let response = std::str::from_utf8(&buf[..bytes_read])
            .expect("TCP connection received invalid UTF-8 character !");

        if response != "<OK>\n" {
            return Err(Error::NotOk);
        }

        stream.write_all("C_OWNER \"overseer\"\n".as_bytes())?;
        let bytes_read = stream.read(&mut buf)?;

        let response = std::str::from_utf8(&buf[..bytes_read])
            .expect("TCP connection received invalid UTF-8 character !");

        if response == "<OK>\n" {
            Ok(Self { stream })
        } else {
            Err(Error::NotOk)
        }
    }

    fn list_interfaces(&mut self) -> Result<Interfaces, Error> {
        self.stream.write_all(b"*/* P_RESERVATION ?\nSYNC\n")?;

        let mut buf = [0u8; 2048];

        let mut interfaces = Interfaces::default();
        loop {
            let bytes_read = self.stream.read(&mut buf)?;

            let stream_input = std::str::from_utf8(&buf[..bytes_read])
                .expect("TCP connection received invalid UTF-8 characters !");

            let lines = stream_input.split('\n').filter(|str| !str.is_empty());

            for line in lines {
                // Attained the SYNC statement, we have finished
                if line == "<SYNC>" {
                    return Ok(interfaces);
                }

                let (module, port, state) = parse_interface_from_line(line)?;

                interfaces
                    .modules
                    .entry(module)
                    .or_default()
                    .insert(port, state);
            }
        }
    }

    fn lock_interface(&mut self, module: u8, port: u8) -> Result<(), Error> {
        let line = format!("{module}/{port} P_RESERVATION RESERVE\n");
        self.send_await_ok(line.as_bytes())
    }

    fn unlock_interface(&mut self, module: u8, port: u8) -> Result<(), Error> {
        let line = format!("{module}/{port} P_RESERVATION RELEASE\n");
        self.send_await_ok(line.as_bytes())
    }

    fn relinquish_interface(&mut self, module: u8, port: u8) -> Result<(), Error> {
        let line = format!("{module}/{port} P_RESERVATION RELINQUISH\n");
        self.send_await_ok(line.as_bytes())
    }

    fn send_await_ok(&mut self, line_as_bytes: &[u8]) -> Result<(), Error> {
        self.stream.write_all(line_as_bytes)?;

        let mut buf = [0u8; 2048];

        let bytes_read = self.stream.read(&mut buf)?;
        let success = std::str::from_utf8(&buf[..bytes_read])
            .expect("TCP connection received invalid UTF-8 charcters !");

        (success == "<OK>\n").then_some(()).ok_or(Error::NotOk)
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

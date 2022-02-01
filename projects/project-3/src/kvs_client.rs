use crate::{Command, Response, Result};
use std::net::SocketAddr;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

/// A type that abstracts the kvs client.
pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    /// Creates a client that connects the server with the given `addr`.
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient { stream })
    }

    /// Sends the given `command` to the server
    pub fn send(&mut self, command: Command) -> Result<Response> {
        let buffer = crate::ser::to_string(&command)?;
        self.stream.write_all(buffer.as_bytes())?;

        let mut response = String::new();
        self.stream.read_to_string(&mut response)?;

        crate::de::from_str(&response)
    }
}

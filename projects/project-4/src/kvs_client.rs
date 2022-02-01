use crate::{Command, Response, Result};

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
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
        self.stream
            .write_all(format!("{}#{}", buffer.len(), buffer).as_bytes())?;

        // Error with shutdown, may cause more unexpected RST
        // see https://stackoverflow.com/questions/70796728/why-does-shutdown-write-in-the-client-cause-the-connection-to-be-closed
        // self.stream.shutdown(std::net::Shutdown::Write)?;

        let mut buffer = Vec::new();
        self.stream.read_to_end(&mut buffer)?;

        crate::de::from_str(std::str::from_utf8(&buffer).unwrap())
    }
}

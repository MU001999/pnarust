use crate::{Command, Response, Result};
use std::net::SocketAddr;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient { stream })
    }

    pub fn send(&mut self, command: Command) -> Result<Response> {
        let buffer = crate::ser::to_string(&command)?;
        self.stream.write_all(buffer.as_bytes())?;

        self.stream.shutdown(Shutdown::Write)?;

        let mut buffer = Vec::new();
        self.stream.read_to_end(&mut buffer)?;

        self.stream.shutdown(Shutdown::Read)?;

        crate::de::from_str(std::str::from_utf8(&buffer).unwrap())
    }
}

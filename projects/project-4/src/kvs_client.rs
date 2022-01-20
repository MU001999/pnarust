use crate::{Command, Response, Result};
use std::net::SocketAddr;
use std::{
    io::{Read, Write},
    net::TcpStream,
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
        self.stream
            .write_all(format!("{}#{}", buffer.len(), buffer).as_bytes())?;

        let mut buffer = Vec::new();
        self.stream.read_to_end(&mut buffer)?;

        crate::de::from_str(std::str::from_utf8(&buffer).unwrap())
    }
}

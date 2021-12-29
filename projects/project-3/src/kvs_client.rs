use crate::{Command, Result};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: String) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient { stream })
    }

    pub fn send(&mut self, command: Command) -> Result<String> {
        let buffer = crate::ser::to_string(&command)?;
        self.stream.write_all(buffer.as_bytes())?;

        let mut res = String::new();
        self.stream.read_to_string(&mut res)?;

        Ok(res)
    }
}

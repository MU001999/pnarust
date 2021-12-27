use crate::{Result, Command};
use std::{net::TcpStream, io::{Read, Write}};

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn connect(addr: String) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient{
            stream
        })
    }

    pub fn send(&mut self, command: Command) -> Result<String> {
        let mut buffer = serde_json::to_vec(&command)?;
        buffer.push(b'#');

        self.stream.write_all(&buffer)?;

        let mut res = String::new();
        self.stream.read_to_string(&mut res)?;

        Ok(res)
    }
}

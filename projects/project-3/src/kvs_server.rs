use crate::{Command, KvsEngine, Result};
use slog::{info, Logger};
use std::{
    io::{BufRead, BufReader, BufWriter, Write, Read},
    net::{TcpListener, TcpStream},
};

pub struct KvsServer<'sv> {
    logger: &'sv Logger,
    engine: &'sv mut dyn KvsEngine,
    addr: String,
}

impl<'sv> KvsServer<'sv> {
    pub fn new(logger: &'sv Logger, engine: &'sv mut dyn KvsEngine, addr: String) -> Result<Self> {
        Ok(KvsServer {
            logger,
            engine,
            addr,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)?;

        for stream in listener.incoming() {
            let stream = stream?;
            info!(self.logger, "accept connection: {:?}", stream.peer_addr());
            self.handle_connection(stream)?;
        }

        Ok(())
    }

    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);

        let mut buffer = [0; 1024];
        let len = reader.read(&mut buffer)?;

        let request = std::str::from_utf8(&buffer[..len]).unwrap();
        info!(self.logger, "received data: {:?}", request);

        let command = crate::de::from_str(request)?;
        info!(self.logger, "received command: {:?}", command);

        match command {
            Command::Set { key, value } => {
                self.engine.set(key, value)?;
                stream.write_all(b"success")?;
            }
            Command::Get { key } => {
                let writer = BufWriter::new(stream);
                let value = self.engine.get(key)?;
                serde_json::to_writer(writer, &value)?;
            }
            Command::Rm { key } => {
                self.engine.remove(key)?;
                stream.write_all(b"success")?;
            }
        }

        Ok(())
    }
}

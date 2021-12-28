use crate::{Result, KvsEngine, Command};
use slog::{info, Logger};
use std::{net::{TcpListener, TcpStream}, io::{BufReader, BufWriter, Write, BufRead}};

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

        let mut buffer = Vec::new();
        reader.read_until(b'#', &mut buffer)?;
        buffer.pop();

        let command = serde_json::from_slice(&buffer[..])?;
        info!(self.logger, "received command: {:?}", command);

        match command {
            Command::Set { key, value} => {
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

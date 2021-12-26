use crate::{Result, KvsEngine, Command};
use slog::{info, Logger};
use std::{net::{TcpListener, TcpStream}, io::{BufReader, BufWriter}};

pub struct KvsServer<'sv> {
    logger: &'sv Logger,
    addr: String,
    engine: &'sv mut dyn KvsEngine,
}

impl<'sv> KvsServer<'sv> {
    pub fn new(logger: &'sv Logger, addr: String, engine: &'sv mut dyn KvsEngine) -> Self {
        KvsServer {
            logger,
            addr,
            engine,
        }
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
        let reader = BufReader::new(&mut stream);

        let command = serde_json::from_reader(reader)?;
        match command {
            Command::Set { key, value} => {
                self.engine.set(key, value)?;
            }
            Command::Get { key } => {
                let writer = BufWriter::new(stream);
                let value = self.engine.get(key)?;
                serde_json::to_writer(writer, &value)?;
            }
            Command::Rm { key } => {
                self.engine.remove(key)?;
            }
        }

        Ok(())
    }
}

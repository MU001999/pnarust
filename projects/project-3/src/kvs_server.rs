use crate::{Command, Response, KvsEngine, Error, Result};
use slog::{info, Logger};
use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream, SocketAddr},
};

pub struct KvsServer<'sv> {
    logger: &'sv Logger,
    engine: &'sv mut dyn KvsEngine,
    addr: SocketAddr,
}

impl<'sv> KvsServer<'sv> {
    pub fn new(logger: &'sv Logger, engine: &'sv mut dyn KvsEngine, addr: SocketAddr) -> Result<Self> {
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
            info!(self.logger, "Accept connection from: {:?}", stream.peer_addr());
            self.handle_connection(stream)?;
        }

        Ok(())
    }

    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);

        let mut buffer = [0; 1024];
        let len = reader.read(&mut buffer)?;

        let request = std::str::from_utf8(&buffer[..len]).unwrap();
        info!(self.logger, "Received data: {:?}", request);

        let command = crate::de::from_str(request)?;
        info!(self.logger, "Received command: {:?}", command);

        let response = match command {
            Command::Set { key, value } => {
                self.engine.set(key.clone(), value.clone())?;
                info!(self.logger, "Set successfully: value {:?} has been set for key {:?}", key, value);
                crate::ser::to_string(&Response::SuccessSet())?
            }
            Command::Get { key } => {
                let value = self.engine.get(key.clone())?;
                info!(self.logger, "Get successfully: value {:?}", value);
                crate::ser::to_string(&Response::SuccessGet(value))?
            }
            Command::Rm { key } => {
                match self.engine.remove(key.clone()) {
                    Ok(()) => {
                        info!(self.logger, "Rm successfully: key {:?}", key);
                        crate::ser::to_string(&Response::SuccessRm())?
                    }
                    Err(Error::KeyNotFound) => {
                        info!(self.logger, "Rm failed: key {:?}", key);
                        crate::ser::to_string(&Response::Fail(String::from("Key not found")))?
                    }
                    Err(e) => return Err(e)
                }
            }
        };

        stream.write_all(response.as_bytes())?;
        info!(self.logger, "Response: {:?}", response);

        Ok(())
    }
}

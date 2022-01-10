use crate::thread_pool::{NaiveThreadPool, SharedQueueThreadPool};
use crate::{thread_pool::ThreadPool, Command, Error, KvsEngine, Response, Result};
use slog::{info, Logger};
use std::{
    io::{BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

#[derive(Clone)]
pub struct KvsServer<E: KvsEngine> {
    logger: Logger,
    engine: E,
    addr: SocketAddr,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(logger: Logger, engine: E, addr: SocketAddr) -> Result<Self> {
        Ok(KvsServer {
            logger,
            engine,
            addr,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        let thd_pool = SharedQueueThreadPool::new(10)?;

        for stream in listener.incoming() {
            let mut stream = stream?;
            info!(
                self.logger,
                "Accept connection from: {:?}",
                stream.peer_addr()
            );

            let mut server = self.clone();
            thd_pool.spawn(move || {
                let command = server.read_command(&stream).unwrap();
                let response = server.process_command(command).unwrap();
                server.respond(&mut stream, response).unwrap();
            });
        }

        Ok(())
    }

    fn read_command(&self, stream: &TcpStream) -> Result<Command> {
        let mut reader = BufReader::new(stream);

        let mut buffer = [0; 1024];
        let len = reader.read(&mut buffer)?;

        let request = std::str::from_utf8(&buffer[..len]).unwrap();
        info!(self.logger, "Received data: {:?}", request);

        let command = crate::de::from_str(request)?;
        info!(self.logger, "Received command: {:?}", command);

        Ok(command)
    }

    fn process_command(&mut self, command: Command) -> Result<String> {
        Ok(match command {
            Command::Set { key, value } => {
                self.engine.set(key.clone(), value.clone())?;
                info!(
                    self.logger,
                    "Set successfully: value {:?} has been set for key {:?}", value, key
                );
                crate::ser::to_string(&Response::SuccessSet())?
            }
            Command::Get { key } => {
                let value = self.engine.get(key)?;
                info!(self.logger, "Get successfully: value {:?}", value);
                crate::ser::to_string(&Response::SuccessGet(value))?
            }
            Command::Rm { key } => match self.engine.remove(key.clone()) {
                Ok(()) => {
                    info!(self.logger, "Rm successfully: key {:?}", key);
                    crate::ser::to_string(&Response::SuccessRm())?
                }
                Err(Error::KeyNotFound) => {
                    info!(self.logger, "Rm failed: key {:?}", key);
                    crate::ser::to_string(&Response::Fail(String::from("Key not found")))?
                }
                Err(e) => return Err(e),
            },
        })
    }

    fn respond(&self, stream: &mut TcpStream, response: String) -> Result<()> {
        stream.write_all(response.as_bytes())?;
        info!(self.logger, "Response: {:?}", response);
        Ok(())
    }
}

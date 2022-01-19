use crate::thread_pool::*;
use crate::{Command, Error, KvsEngine, Response, Result};
use slog::{info, Logger};
use std::{
    io::{BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
};

pub struct KvsServer<E: KvsEngine, T: ThreadPool> {
    logger: Logger,
    listener: TcpListener,
    engine: E,
    thread_pool: T,
}

impl<E: KvsEngine, T: ThreadPool> KvsServer<E, T> {
    pub fn new(logger: Logger, addr: SocketAddr, engine: E, thread_pool: T) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(KvsServer {
            logger,
            listener,
            engine,
            thread_pool,
        })
    }

    /// Run the server with given number of conns,
    /// run without existing if conns is none.
    pub fn run(&mut self, conns: Option<usize>) -> Result<()> {
        let mut conns_cnt = 0;
        for stream in self.listener.incoming() {
            let mut stream = stream?;
            info!(
                self.logger,
                "Accept connection from: {:?}",
                stream.peer_addr()
            );

            let logger = self.logger.clone();
            let engine = self.engine.clone();
            self.thread_pool.spawn(move || {
                while let Some(command) = read_command(&logger, &stream).unwrap() {
                    let response = process_command(&logger, engine.clone(), command).unwrap();
                    respond(&logger, &mut stream, response).unwrap();
                }
                stream.shutdown(Shutdown::Both).unwrap();
            });

            conns_cnt += 1;
            if let Some(conns) = conns {
                if conns_cnt >= conns {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }
}

fn read_command(logger: &Logger, stream: &TcpStream) -> Result<Option<Command>> {
    let mut reader = BufReader::new(stream);

    let mut buffer = [0; 1024];
    let len = reader.read(&mut buffer)?;

    if len != 0 {
        let request = std::str::from_utf8(&buffer[..len]).unwrap();
        info!(logger, "Received data: {:?}", request);

        let command = crate::de::from_str(request)?;
        info!(logger, "Received command: {:?}", command);

        Ok(Some(command))
    } else {
        Ok(None)
    }
}

fn process_command(logger: &Logger, engine: impl KvsEngine, command: Command) -> Result<String> {
    Ok(match command {
        Command::Set { key, value } => {
            engine.set(key.clone(), value.clone())?;
            info!(
                logger,
                "Set successfully: value {:?} has been set for key {:?}", value, key
            );
            crate::ser::to_string(&Response::SuccessSet())?
        }
        Command::Get { key } => {
            let value = engine.get(key)?;
            info!(logger, "Get successfully: value {:?}", value);
            crate::ser::to_string(&Response::SuccessGet(value))?
        }
        Command::Rm { key } => match engine.remove(key.clone()) {
            Ok(()) => {
                info!(logger, "Rm successfully: key {:?}", key);
                crate::ser::to_string(&Response::SuccessRm())?
            }
            Err(Error::KeyNotFound) => {
                info!(logger, "Rm failed: key {:?}", key);
                crate::ser::to_string(&Response::Fail(String::from("Key not found")))?
            }
            Err(e) => return Err(e),
        },
    })
}

fn respond(logger: &Logger, stream: &mut TcpStream, response: String) -> Result<()> {
    stream.write_all(response.as_bytes())?;
    info!(logger, "Response: {:?}", response);
    Ok(())
}

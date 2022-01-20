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

    /// Run the server with given number of tasks,
    /// run without existing if tasks is none.
    pub fn run(&mut self, tasks: Option<usize>) -> Result<()> {
        let mut tasks_cnt = 0;
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
                let command = read_command(&logger, &stream).unwrap();
                let response = process_command(engine.clone(), command).unwrap();
                respond(&logger, &mut stream, response).unwrap();
            });

            tasks_cnt += 1;
            if let Some(tasks) = tasks {
                if tasks_cnt >= tasks {
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

fn read_command(logger: &Logger, stream: &TcpStream) -> Result<Command> {
    let mut reader = BufReader::new(stream);

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    stream.shutdown(Shutdown::Read)?;

    let request = std::str::from_utf8(&buffer).unwrap();
    let command = crate::de::from_str(request)?;
    info!(logger, "Received command: {:?}", command);

    Ok(command)
}

fn process_command(engine: impl KvsEngine, command: Command) -> Result<String> {
    Ok(match command {
        Command::Set { key, value } => {
            engine.set(key, value)?;
            crate::ser::to_string(&Response::SuccessSet())?
        }
        Command::Get { key } => {
            let value = engine.get(key)?;
            crate::ser::to_string(&Response::SuccessGet(value))?
        }
        Command::Rm { key } => match engine.remove(key) {
            Ok(()) => crate::ser::to_string(&Response::SuccessRm())?,
            Err(Error::KeyNotFound) => {
                crate::ser::to_string(&Response::Fail(String::from("Key not found")))?
            }
            Err(e) => return Err(e),
        },
    })
}

fn respond(logger: &Logger, stream: &mut TcpStream, response: String) -> Result<()> {
    stream.write_all(response.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;

    info!(logger, "Response: {:?}", response);
    Ok(())
}

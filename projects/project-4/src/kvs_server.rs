use crate::{kvs_engine::KvsEngine, thread_pool::*, Command, Error, Response, Result};

use slog::{info, Logger};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
};

/// A type that abstracts the kvs server.
pub struct KvsServer<E: KvsEngine, T: ThreadPool> {
    logger: Logger,
    listener: TcpListener,
    engine: E,
    thread_pool: T,
}

impl<E: KvsEngine, T: ThreadPool> KvsServer<E, T> {
    /// Creates a server with a logger, a listening address, a store engine and a thread pool.
    pub fn new(logger: Logger, addr: SocketAddr, engine: E, thread_pool: T) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(KvsServer {
            logger,
            listener,
            engine,
            thread_pool,
        })
    }

    /// Starts receiving requests and replying responses.
    ///
    /// Quits after processing `N` tasks with `Some(N)` as `tasks`,
    /// or keeps running if `tasks` is `None`.
    ///
    /// NOTE: the tasks is designed for benchmarks
    pub fn run(&mut self, tasks: Option<usize>) -> Result<()> {
        let mut tasks_cnt = 0;
        for stream in self.listener.incoming() {
            let mut stream = stream?;
            info!(
                self.logger,
                "Accept connection from: {:?}",
                stream.peer_addr()?
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

    // Gets the local address of the server
    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }
}

// reads one command from the stream
fn read_command(logger: &Logger, stream: &TcpStream) -> Result<Command> {
    let mut reader = BufReader::new(stream);

    let mut buffer = Vec::new();
    let len = reader.read_until(b'#', &mut buffer)?;

    let len: usize = std::str::from_utf8(&buffer[0..len - 1])
        .unwrap()
        .parse()
        .unwrap();
    let mut buffer = [0; 1024];
    reader.read_exact(&mut buffer[0..len])?;

    let request = std::str::from_utf8(&buffer[0..len]).unwrap();
    let command = crate::de::from_str(request)?;
    info!(logger, "Received command: {:?}", command);

    Ok(command)
}

// processes a command in the given store engine and returns the response string
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

// responds to the stream with the given response string
fn respond(logger: &Logger, stream: &mut TcpStream, response: String) -> Result<()> {
    stream.write_all(response.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;

    info!(logger, "Response: {:?}", response);
    Ok(())
}

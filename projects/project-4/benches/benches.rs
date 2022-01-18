use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::{thread_pool::*, Command, KvsClient, Response};
use kvs::{KvStore, KvsEngine, KvsServer, SledKvsEngine};
use sloggers::null::NullLoggerBuilder;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::Build;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use tempfile::TempDir;

pub fn criterion_benchmark(c: &mut Criterion) {
    let ncpus = num_cpus::get();
    let mut inputs = vec![1];
    for n in 1..=ncpus {
        inputs.push(n * 2);
    }

    const NCONN: usize = 2;

    c.bench_function_over_inputs(
        "write_queued_kvstore",
        |b, &threads| {
            let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
            b.iter_batched(
                || {
                    let keys: Vec<String> = (0..NCONN).map(|n| format!("{:0>8}", n)).collect();

                    let mut builder = TerminalLoggerBuilder::new();
                    builder.destination(Destination::Stderr);
                    // let builder = NullLoggerBuilder;

                    let temp_dir =
                        TempDir::new().expect("unable to create temporary working directory");
                    let path = temp_dir.path().join("db.kvs");

                    let logger = builder.build().unwrap();
                    let engine = KvStore::open(path.clone()).unwrap();
                    let thread_pool = SharedQueueThreadPool::new(threads).unwrap();

                    let server = std::thread::spawn(move || {
                        KvsServer::new(logger, addr, engine, thread_pool)
                            .unwrap()
                            .run(Some(NCONN))
                            .unwrap();
                    });
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    (server, keys)
                },
                |(server, keys)| {
                    for key in keys {
                        let mut client = KvsClient::connect(addr).unwrap();
                        let resp = client
                            .send(Command::Set {
                                key,
                                value: String::from("value"),
                            })
                            .unwrap();
                        assert_eq!(resp, Response::SuccessSet());
                    }
                    server.join().unwrap();
                },
                BatchSize::SmallInput,
            );
        },
        inputs.clone(),
    );

    c.bench_function_over_inputs(
        "read_queued_kvstore",
        |b, &threads| {
            let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
            b.iter_batched(
                || {
                    let keys: Vec<String> = (0..NCONN).map(|n| format!("{:0>8}", n)).collect();

                    let builder = NullLoggerBuilder;

                    let temp_dir =
                        TempDir::new().expect("unable to create temporary working directory");
                    let path = temp_dir.path().join("db.kvs");

                    let logger = builder.build().unwrap();
                    let engine = KvStore::open(path.clone()).unwrap();
                    let thread_pool = SharedQueueThreadPool::new(threads).unwrap();

                    let server = std::thread::spawn(move || {
                        KvsServer::new(logger, addr, engine, thread_pool)
                            .unwrap()
                            .run(Some(2 * NCONN))
                            .unwrap();
                    });
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    for key in &keys {
                        let mut client = KvsClient::connect(addr).unwrap();
                        let resp = client
                            .send(Command::Set {
                                key: key.clone(),
                                value: String::from("value"),
                            })
                            .unwrap();
                        assert_eq!(resp, Response::SuccessSet());
                    }

                    let clients = SharedQueueThreadPool::new(NCONN).unwrap();

                    (server, clients, keys)
                },
                |(server, clients, keys)| {
                    let (sender, receiver) = channel();
                    for key in keys {
                        let addr = addr.clone();
                        let sender = sender.clone();
                        clients.spawn(move || {
                            let mut client = KvsClient::connect(addr).unwrap();
                            let resp = client.send(Command::Get { key }).unwrap();
                            assert_eq!(resp, Response::SuccessGet(Some(String::from("value"))));
                            sender.send(1).unwrap();
                        })
                    }

                    for _ in 0..NCONN {
                        assert_eq!(receiver.recv().unwrap(), 1);
                    }
                    server.join().unwrap();
                },
                BatchSize::SmallInput,
            );
        },
        inputs.clone(),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

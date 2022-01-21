use criterion::{criterion_group, criterion_main, BatchSize, Bencher, BenchmarkId, Criterion};
use kvs::{thread_pool::*, Command, KvsClient, Response};
use kvs::{KvStore, KvsEngine, KvsServer, SledKvsEngine};
use sloggers::null::NullLoggerBuilder;
// use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::Build;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use tempfile::TempDir;

const NTASK: usize = 1000;

pub fn criterion_write(c: &mut Criterion) {
    let ncpus = num_cpus::get();
    let mut inputs = vec![1];
    for n in 1..=ncpus {
        inputs.push(n * 2);
    }

    let mut group = c.benchmark_group("write");
    group.sample_size(10);

    for input in &inputs {
        group.bench_with_input(
            BenchmarkId::new("write_queued_kvstore", input),
            input,
            write_function::<KvStore, SharedQueueThreadPool>,
        );
        group.bench_with_input(
            BenchmarkId::new("write_rayon_kvstore", input),
            input,
            write_function::<KvStore, RayonThreadPool>,
        );
        group.bench_with_input(
            BenchmarkId::new("write_rayon_sledkvengine", input),
            input,
            write_function::<SledKvsEngine, RayonThreadPool>,
        );
    }
}

pub fn criterion_read(c: &mut Criterion) {
    let ncpus = num_cpus::get();
    let mut inputs = vec![1];
    for n in 1..=ncpus {
        inputs.push(n * 2);
    }

    let mut group = c.benchmark_group("read");
    group.sample_size(10);

    for input in &inputs {
        group.bench_with_input(
            BenchmarkId::new("read_queued_kvstore", input),
            input,
            read_function::<KvStore, SharedQueueThreadPool>,
        );
        group.bench_with_input(
            BenchmarkId::new("read_rayon_kvstore", input),
            input,
            read_function::<KvStore, RayonThreadPool>,
        );
        group.bench_with_input(
            BenchmarkId::new("read_rayon_sledkvengine", input),
            input,
            read_function::<SledKvsEngine, RayonThreadPool>,
        );
    }
}

criterion_group!(benches, criterion_write, criterion_read);
criterion_main!(benches);

fn write_function<E, T>(b: &mut Bencher, &threads: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    let keys: Vec<String> = (0..NTASK).map(|n| format!("{:0>8}", n)).collect();
    b.iter_batched(
        || {
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            // let mut builder = TerminalLoggerBuilder::new();
            // builder.destination(Destination::Stderr);
            // let logger = builder.build().unwrap();

            let logger = NullLoggerBuilder.build().unwrap();
            let engine = E::open(path.clone()).unwrap();
            let thread_pool = T::new(threads).unwrap();

            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let mut server = KvsServer::new(logger, addr, engine, thread_pool).unwrap();
            let addr = server.local_addr();

            let server = std::thread::spawn(move || {
                server.run(Some(NTASK)).unwrap();
            });
            std::thread::sleep(std::time::Duration::from_secs(1));

            let clients = T::new(num_cpus::get()).unwrap();
            (server, clients, keys.clone(), addr, temp_dir)
        },
        |(server, clients, keys, addr, _)| {
            let (sender, receiver) = channel();
            for key in keys.into_iter() {
                let sender = sender.clone();
                clients.spawn(move || {
                    let mut client = KvsClient::connect(addr).unwrap();
                    let resp = client
                        .send(Command::Set {
                            key,
                            value: String::from("value"),
                        })
                        .unwrap();
                    assert_eq!(resp, Response::SuccessSet());
                    while sender.send(1).is_err() {}
                });
            }

            for _ in 0..NTASK {
                assert_eq!(receiver.recv().unwrap(), 1);
            }
            server.join().unwrap();
        },
        BatchSize::PerIteration,
    );
}

fn read_function<E, T>(b: &mut Bencher, &threads: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    let keys: Vec<String> = (0..NTASK).map(|n| format!("{:0>8}", n)).collect();
    b.iter_batched(
        || {
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            // let mut builder = TerminalLoggerBuilder::new();
            // builder.destination(Destination::Stderr);
            // let logger = builder.build().unwrap();

            let logger = NullLoggerBuilder.build().unwrap();
            let engine = E::open(path.clone()).unwrap();
            let thread_pool = T::new(threads).unwrap();

            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let mut server = KvsServer::new(logger, addr, engine, thread_pool).unwrap();
            let addr = server.local_addr();

            let server = std::thread::spawn(move || {
                server.run(Some(NTASK * 2)).unwrap();
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

            let clients = T::new(NTASK).unwrap();
            (server, clients, keys.clone(), addr, temp_dir)
        },
        |(server, clients, keys, addr, _)| {
            let (sender, receiver) = channel();
            for key in keys {
                let sender = sender.clone();
                clients.spawn(move || {
                    let mut client = KvsClient::connect(addr).unwrap();
                    let resp = client.send(Command::Get { key }).unwrap();
                    assert_eq!(resp, Response::SuccessGet(Some(String::from("value"))));
                    while sender.send(1).is_err() {}
                })
            }

            for _ in 0..NTASK {
                assert_eq!(receiver.recv().unwrap(), 1);
            }
            server.join().unwrap();
        },
        BatchSize::PerIteration,
    );
}

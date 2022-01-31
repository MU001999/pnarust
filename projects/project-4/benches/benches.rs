use kvs::{kvs_engine::*, thread_pool::*, Command, KvsClient, KvsServer, Response};

use criterion::{criterion_group, criterion_main, BatchSize, Bencher, BenchmarkId, Criterion};
use sloggers::{null::NullLoggerBuilder, Build};
use std::{net::SocketAddr, sync::mpsc::channel};
use tempfile::TempDir;

// one task for one thread to set/get
const NTASK: usize = 1000;

// constructs the inputs list [1, 2, 4, 6, ..., ncpus * 2]
fn construct_inputs() -> Vec<usize> {
    let ncpus = num_cpus::get();
    let mut inputs = vec![1];
    for n in 1..=ncpus {
        inputs.push(n * 2);
    }
    inputs
}

// benchmarks write operations with different threads (1, 2, 4, ..., ncpus * 2)
// sample size is set to 10 for short runtime
pub fn criterion_write(c: &mut Criterion) {
    let inputs = construct_inputs();

    // set the sample size to 10
    let mut group = c.benchmark_group("write");
    group.sample_size(10);

    // benchmark different couples of KvsEngine and ThreadPool
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

// benchmarks read operations with different threads (1, 2, 4, ..., ncpus * 2)
// sample size is set to 10 for short runtime
pub fn criterion_read(c: &mut Criterion) {
    let inputs = construct_inputs();

    // sets the sample size to 10
    let mut group = c.benchmark_group("read");
    group.sample_size(10);

    // benchmarks different couples of KvsEngine and ThreadPool
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

// the actual benchmark function for write-operations
fn write_function<E, T>(b: &mut Bencher, &threads: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    // key-values are [(00000000, value), ..., (00000999, value)]
    let keys: Vec<String> = (0..NTASK).map(|n| format!("{:0>8}", n)).collect();
    b.iter_batched(
        || {
            // inits the temp dir and path to store,
            // where the temp_dir needs to be passed to the routine function
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            // inits the logger, engine and thread_pool for the server
            let logger = NullLoggerBuilder.build().unwrap();
            let engine = E::open(path.clone()).unwrap();
            let thread_pool = T::new(threads).unwrap();

            // inits the server with an available address
            // and gets the local addr for the connections from clients later
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let mut server = KvsServer::new(logger, addr, engine, thread_pool).unwrap();
            let addr = server.local_addr();

            // the server runs in an other thread and stops after receiving NTASK tasks
            let server = std::thread::spawn(move || {
                server.run(Some(NTASK)).unwrap();
            });
            // sleeps 1s to make sure the server thread has been ready
            std::thread::sleep(std::time::Duration::from_secs(1));

            // inits the thread_pool for clients
            let clients = T::new(num_cpus::get()).unwrap();
            // passes the temp_dir to hold the temp dir
            (server, clients, keys.clone(), addr, temp_dir)
        },
        |(server, clients, keys, addr, _)| {
            // inits the sender and receiver to make sure all tasks have been done
            let (sender, receiver) = channel();
            for key in keys.into_iter() {
                let sender = sender.clone();
                // every client just sends one command to set corresponding key and value
                clients.spawn(move || {
                    let mut client = KvsClient::connect(addr).unwrap();
                    let resp = client
                        .send(Command::Set {
                            key,
                            value: String::from("value"),
                        })
                        .unwrap();
                    // asserts the response is as expected
                    assert_eq!(resp, Response::SuccessSet());
                    // sends 1 for representing this task has been done
                    while sender.send(1).is_err() {}
                });
            }

            // waits for all tasks
            for _ in 0..NTASK {
                assert_eq!(receiver.recv().unwrap(), 1);
            }
            // waits for the server thread
            server.join().unwrap();
        },
        BatchSize::PerIteration,
    );
}

// the actual benchmark function for read-operations
fn read_function<E, T>(b: &mut Bencher, &threads: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    // key-values are [(00000000, value), ..., (00000999, value)]
    let keys: Vec<String> = (0..NTASK).map(|n| format!("{:0>8}", n)).collect();
    b.iter_batched(
        || {
            // inits the temp dir and path to store,
            // where the temp_dir needs to be passed to the routine function
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            // inits the logger, engine and thread_pool for the server
            let logger = NullLoggerBuilder.build().unwrap();
            let engine = E::open(path.clone()).unwrap();
            let thread_pool = T::new(threads).unwrap();

            // inits the server with an available address
            // and gets the local addr for the connections from clients later
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let mut server = KvsServer::new(logger, addr, engine, thread_pool).unwrap();
            let addr = server.local_addr();

            // the server runs in an other thread and stops after receiving NTASK * 2 tasks,
            // first NTASK tasks to set and second NTASK tasks to get
            let server = std::thread::spawn(move || {
                server.run(Some(NTASK * 2)).unwrap();
            });
            // sleeps 1s to make sure the server thread has been ready
            std::thread::sleep(std::time::Duration::from_secs(1));

            // sets all key-values before the benchmark
            for key in &keys {
                let mut client = KvsClient::connect(addr).unwrap();
                let resp = client
                    .send(Command::Set {
                        key: key.clone(),
                        value: String::from("value"),
                    })
                    .unwrap();
                // asserts the response as expected
                assert_eq!(resp, Response::SuccessSet());
            }

            // inits the thread_pool for clients
            let clients = T::new(NTASK).unwrap();
            // passes the temp_dir to hold the temp dir
            (server, clients, keys.clone(), addr, temp_dir)
        },
        |(server, clients, keys, addr, _)| {
            // inits the sender and receiver to make sure all tasks have been done
            let (sender, receiver) = channel();
            for key in keys {
                let sender = sender.clone();
                // every client just sends one command to get corresponding value
                clients.spawn(move || {
                    let mut client = KvsClient::connect(addr).unwrap();
                    let resp = client.send(Command::Get { key }).unwrap();
                    // asserts the response is as expected
                    assert_eq!(resp, Response::SuccessGet(Some(String::from("value"))));
                    // sends 1 for representing this task has been done
                    while sender.send(1).is_err() {}
                })
            }

            // waits for all tasks
            for _ in 0..NTASK {
                assert_eq!(receiver.recv().unwrap(), 1);
            }
            // waits for the server thread
            server.join().unwrap();
        },
        BatchSize::PerIteration,
    );
}

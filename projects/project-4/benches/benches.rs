use criterion::{criterion_group, criterion_main, Criterion};
use kvs::{thread_pool::*, Command, KvsClient, Response};
use kvs::{KvStore, KvsEngine, KvsServer, SledKvsEngine};
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

    c.bench_function_over_inputs(
        "write_queued_kvstore",
        move |b, &threads| {
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            let mut builder = TerminalLoggerBuilder::new();
            builder.destination(Destination::Stderr);

            let logger = builder.build().unwrap();
            let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
            let engine = KvStore::open(path.clone()).unwrap();
            let thread_pool = SharedQueueThreadPool::new(threads).unwrap();

            let server = std::thread::spawn(move || {
                KvsServer::new(logger, addr, engine, thread_pool)
                    .unwrap()
                    .run(Some(1000))
                    .unwrap();
            });
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut client = KvsClient::connect(addr).unwrap();
            let keys: Vec<String> = (0..1000).map(|n| format!("{:0>8}", n)).collect();
            for key in keys {
                b.iter(|| {
                    let resp = client
                        .send(Command::Set {
                            key: key.clone(),
                            value: String::from("value"),
                        })
                        .unwrap();
                    assert_eq!(resp, Response::SuccessSet());
                })
            }

            server.join().unwrap();
        },
        inputs.clone(),
    );

    c.bench_function_over_inputs(
        "read_queued_kvstore",
        |b, &threads| {
            let temp_dir = TempDir::new().expect("unable to create temporary working directory");
            let path = temp_dir.path().join("db.kvs");

            let mut builder = TerminalLoggerBuilder::new();
            builder.destination(Destination::Stderr);

            let logger = builder.build().unwrap();
            let addr: SocketAddr = "127.0.0.1:4000".parse().unwrap();
            let engine = KvStore::open(path.clone()).unwrap();
            let thread_pool = SharedQueueThreadPool::new(threads).unwrap();

            let server = std::thread::spawn(move || {
                KvsServer::new(logger, addr, engine, thread_pool)
                    .unwrap()
                    .run(Some(1000))
                    .unwrap();
            });
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut client = KvsClient::connect(addr).unwrap();
            let keys: Vec<String> = (0..1000).map(|n| format!("{:0>8}", n)).collect();
            for key in &keys {
                let resp = client
                    .send(Command::Set {
                        key: key.clone(),
                        value: String::from("value"),
                    })
                    .unwrap();
                assert_eq!(resp, Response::SuccessSet());
            }

            let clients = SharedQueueThreadPool::new(1000).unwrap();
            b.iter(|| {
                let (sender, receiver) = channel();
                for key in &keys {
                    let addr = addr.clone();
                    let key = key.clone();
                    let sender = sender.clone();
                    clients.spawn(move || {
                        let mut client = KvsClient::connect(addr).unwrap();
                        let resp = client.send(Command::Get { key }).unwrap();
                        assert_eq!(resp, Response::SuccessGet(Some(String::from("value"))));
                        sender.send(1).unwrap();
                    })
                }

                for _ in 0..1000 {
                    assert_eq!(receiver.recv().unwrap(), 1);
                }
            });

            server.join().unwrap();
        },
        inputs.clone(),
    );

    /*
       let temp_dir = TempDir::new().expect("unable to create temporary working directory");

       let mut db_kvs =
           KvStore::open(temp_dir.path().join("db.kvs")).expect("unable to open the KvStore");
       let mut db_sled = SledKvsEngine::open(temp_dir.path().join("db.sled"))
           .expect("unable to open the Sled Engine");

       let mut key_vals: Vec<(String, String)> = (0..100)
           .map(|_| (generate_str(1, 100000), generate_str(1, 100000)))
           .collect();

       c.bench_function("kvs_write", |b| {
           for (key, value) in &key_vals {
               let (key, value) = (key.as_str(), value.as_str());
               b.iter(|| {
                   db_kvs
                       .set(key.to_owned(), value.to_owned())
                       .expect("unable to set");
               })
           }
       });

       c.bench_function("sled_write", |b| {
           for (key, value) in &key_vals {
               let (key, value) = (key.as_str(), value.as_str());
               b.iter(|| {
                   db_sled
                       .set(key.to_owned(), value.to_owned())
                       .expect("unable to set");
               })
           }
       });

       let mut rng = rand::thread_rng();
       c.bench_function("kvs_read", |b| {
           for _ in 0..10 {
               key_vals.shuffle(&mut rng);
               for (key, value) in &key_vals {
                   let (key, value) = (key.as_str(), value.as_str());
                   b.iter(|| {
                       let res = db_kvs.get(key.to_owned()).expect("unable to set");
                       assert_eq!(res.unwrap(), value);
                   });
               }
           }
       });

       c.bench_function("sled_read", |b| {
           for _ in 0..10 {
               key_vals.shuffle(&mut rng);
               for (key, value) in &key_vals {
                   let (key, value) = (key.as_str(), value.as_str());
                   b.iter(|| {
                       let res = db_sled.get(key.to_owned()).expect("unable to set");
                       assert_eq!(res.unwrap(), value);
                   });
               }
           }
       });
    */
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

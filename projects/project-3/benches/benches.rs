use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::Rng;
use tempfile::TempDir;

fn generate_str(min: usize, max: usize) -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(min..=max);

    let mut result = String::with_capacity(length);
    for _ in 0..length {
        result.push(char::from(rng.sample(Alphanumeric)));
    }
    result
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let key_vals: Vec<(String, String)> = (0..100)
        .map(|_| (generate_str(1, 100000), generate_str(1, 100000)))
        .collect();

    c.bench_function("kvs_write", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let db = KvStore::open(temp_dir.path().join("db.kvs")).unwrap();

                (db, key_vals.clone(), temp_dir)
            },
            |(mut db, key_vals, _)| {
                for (key, value) in key_vals {
                    db.set(key, value).expect("unable to set");
                }
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("sled_write", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let db = SledKvsEngine::open(temp_dir.path().join("db.sled")).unwrap();

                (db, key_vals.clone(), temp_dir)
            },
            |(mut db, key_vals, _)| {
                for (key, value) in key_vals {
                    db.set(key, value).expect("unable to set");
                }
            },
            BatchSize::SmallInput,
        );
    });

    let mut rng = rand::thread_rng();
    c.bench_function("kvs_read", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let mut db = KvStore::open(temp_dir.path().join("db.kvs")).unwrap();

                for (key, value) in &key_vals {
                    db.set(key.to_owned(), value.to_owned()).expect("unable to set");
                }

                (db, key_vals.clone(), temp_dir)
            },
            |(mut db, mut key_vals, _)| {
                for _ in 0..10 {
                    key_vals.shuffle(&mut rng);
                    for (key, value) in &key_vals {
                        let res = db.get(key.to_owned()).unwrap();
                        assert_eq!(res.unwrap(), value.as_str());
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("sled_read", |b| {
        b.iter_batched(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                let mut db = SledKvsEngine::open(temp_dir.path().join("db.sled")).unwrap();

                for (key, value) in &key_vals {
                    db.set(key.to_owned(), value.to_owned()).expect("unable to set");
                }

                (db, key_vals.clone(), temp_dir)
            },
            |(mut db, mut key_vals, _)| {
                for _ in 0..10 {
                    key_vals.shuffle(&mut rng);
                    for (key, value) in &key_vals {
                        let res = db.get(key.to_owned()).unwrap();
                        assert_eq!(res.unwrap(), value.as_str());
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

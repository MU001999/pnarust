use criterion::{criterion_group, criterion_main, Criterion};
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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

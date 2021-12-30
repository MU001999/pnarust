use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kvs::{KvsEngine, KvStore, SledKvsEngine};
use tempfile::TempDir;
use rand::Rng;

pub fn criterion_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut kvstore = KvStore::open(temp_dir.path()).expect("unable to open the KvStore");
    let mut sledkvs = SledKvsEngine::open(temp_dir.path()).expect("unable to open the Sled Engine");

    let mut rng = rand::thread_rng();
    let key_vals: Vec<(i32, i32)> = (0..100).map(|_| (rng.gen_range(0..100000), rng.gen_range(0..100000))).collect();

    c.bench_function("kvs_write", |b| {
        for (key, value) in &key_vals {
            b.iter(|| {
                kvstore.set(key.to_string(), value.to_string()).expect("unable to set");
            })
        }
    });

    c.bench_function("sled_write", |b| {
        for (key, value) in &key_vals {
            b.iter(|| {
                sledkvs.set(key.to_string(), value.to_string()).expect("unable to set");
            })
        }
    });

    c.bench_function("kvs_read", |b| {
        b.iter(|| {

        })
    });

    c.bench_function("sled_read", |b| {
        b.iter(|| {

        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

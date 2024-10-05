use criterion::{criterion_group, criterion_main, Criterion};
use reth_provider::{StateProvider, StateProviderFactory};
use rethdb_dexsync::univ2::{UniV2Factory, UNI_V2_FACTORY};
use rethdb_dexsync::utils::init_db_read_only;
use std::path::Path;
use std::time::Duration;

fn load_pairs<T: StateProvider>(provider: T) {
    UniV2Factory::load_pairs(provider, UNI_V2_FACTORY, None).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let db_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata").join("univ2-test-db");
    println!("Path: {:?}", db_path);
    let factory = init_db_read_only(&db_path).unwrap();
    let mut group = c.benchmark_group("load univ2 group");
    group.sample_size(10).warm_up_time(Duration::from_secs(10));
    group.bench_function("load univ2", |b| b.iter(|| load_pairs(factory.latest().unwrap())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use devutils::all_utils;

fn benchmark_utils(c: &mut Criterion) {
    let base64_payload = vec!["Hello World".to_string()];
    
    c.bench_function("base64_encode", |b| {
        b.iter(|| {
            all_utils::run_all_utils_cmd("base64", &base64_payload)
        })
    });
    
    let hash_payload = vec!["SuperSecretHashPayload123".to_string()];
    
    c.bench_function("sha256", |b| {
        b.iter(|| {
            all_utils::run_all_utils_cmd("sha256", &hash_payload)
        })
    });
}

criterion_group!(benches, benchmark_utils);
criterion_main!(benches);

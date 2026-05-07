use criterion::{criterion_group, criterion_main, Criterion};
use devutils::search::FileSearch;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn benchmark_search(c: &mut Criterion) {
    // Create a temporary directory structure for benchmarking
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    
    // Create 100 mock files
    for i in 0..100 {
        let file_path = dir_path.join(format!("test_file_{}.rs", i));
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "fn test_{}() {{ println!(\"Hello World\"); }}", i).unwrap();
    }
    
    let target = dir_path.to_str().unwrap().to_string();
    
    c.bench_function("search_parallel", |b| {
        b.iter(|| {
            let search = FileSearch::new(&target);
            search.find("test_file", None, true)
        })
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);

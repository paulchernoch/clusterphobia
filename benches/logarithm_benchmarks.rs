use criterion::{black_box, criterion_group, criterion_main, Criterion};
use clusterphobia::clustering::logarithm::{log_ratio, log_ratio_small};

fn log_ratio_various() {
    let mut sum = 0.0;
    for numerator in 1..1000_u64 {
        for denominator in 1..10_u64 {
            let approximate_log = log_ratio(black_box(numerator), denominator);
            sum += approximate_log;
        }
    }
    let s = format!("sum = {}", sum);
    assert!(s.len() > 0);
}

fn library_log_various() {
    let mut sum = 0.0;
    for numerator in 1..1000_u64 {
        for denominator in 1..10_u64 {
            let library_log = (black_box(numerator) as f64 / denominator as f64).ln();
            sum += library_log;
        }
    }
    let s = format!("sum = {}", sum);
    assert!(s.len() > 0);
}

/// Log of Ratios that are between one and two.
fn log_ratio_small_10k() {
    let mut sum = 0.0;
    for numerator in 10000..20000_u64 {
        let denominator = 10000;
        let approximate_log = log_ratio_small(black_box(numerator), denominator);
        sum += approximate_log;
    }
    let s = format!("sum = {}", sum);
    assert!(s.len() > 0);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("10k log_ratio for small domain (1..2)", |b| b.iter(|| log_ratio_small_10k()));
    c.bench_function("Many logarithms using log_ratio", |b| b.iter(|| log_ratio_various()));
    c.bench_function("Many logarithms using std library ln", |b| b.iter(|| library_log_various()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

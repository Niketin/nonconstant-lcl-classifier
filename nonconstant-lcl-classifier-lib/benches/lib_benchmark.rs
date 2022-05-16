use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nonconstant_lcl_classifier_lib::{caches::LclProblemSqliteCache, Configurations, LclProblem};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate problems");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10))
        .bench_function("gen problem 3 2 3", |b| {
            b.iter(|| {
                LclProblem::get_or_generate_normalized::<LclProblemSqliteCache>(
                    black_box(3),
                    black_box(2),
                    black_box(3),
                    None,
                )
            })
        });
    group.finish();

    let mut group = c.benchmark_group("Generate powersets");
    group.bench_function("gen powerset 3 2", |b| {
        b.iter(|| Configurations::generate_powerset(black_box(4), black_box(3)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

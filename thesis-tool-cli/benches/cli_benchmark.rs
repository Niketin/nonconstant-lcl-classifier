use criterion::{black_box, criterion_group, criterion_main, Criterion};
use thesis_tool_cli_lib::utils::create_graphs;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate graphs");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10))
        .bench_function("gen graphs 1 16 3 3", |b| {
            b.iter(|| {
                create_graphs(
                    black_box(1),
                    black_box(16),
                    black_box(3),
                    black_box(3),
                )
            })
        });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

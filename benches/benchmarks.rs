//! Benchmarks for dbfast
//! Generated on 2025-09-10

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dbfast::QueryBuilder;

fn bench_query_builder_simple(c: &mut Criterion) {
    c.bench_function("query_builder_simple", |b| {
        b.iter(|| {
            QueryBuilder::new()
                .where_clause(black_box("id = 1"))
                .build()
        });
    });
}

fn bench_query_builder_complex(c: &mut Criterion) {
    c.bench_function("query_builder_complex", |b| {
        b.iter(|| {
            QueryBuilder::new()
                .where_clause(black_box("id = 1"))
                .where_clause(black_box("name = 'test'"))
                .where_clause(black_box("active = true"))
                .param(black_box("param1".to_string()))
                .param(black_box("param2".to_string()))
                .build()
        });
    });
}

criterion_group!(
    benches,
    bench_query_builder_simple,
    bench_query_builder_complex
);
criterion_main!(benches);

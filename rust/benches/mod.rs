use criterion::{criterion_group, criterion_main, Criterion};

pub fn collect(c: &mut Criterion) {

    c.bench_function("collect", |b| b.iter(|| {
        (0..1000).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
            .iter().map(|v| v + 1).collect::<Vec<i32>>()
    }));


    c.bench_function("map", |b| b.iter(|| {
        (0..1000).collect::<Vec<i32>>().iter()
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .map(|v| v + 1)
            .collect::<Vec<i32>>()
    }));
}


criterion_group!(benches, collect);
criterion_main!(benches);
use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::outcomespace_size_benchmark::benches,
    benchmarks::retry_benchmark::benches,
    benchmarks::timingchannel_benchmark::benches,
}
//! Compares the performance of all known tracing options to our own.
//!
//! Analysis 2024-03-10:
//!
//!     Our version is between 7 and 9 times faster than the `tracing-subscriber` versions, both for the OVERHEAD
//! and the THROUGHPUT cases.
//!
//!     Median values:
//!
//!     baseline method call:             4.4160 ns
//!     `tracing-stackdriver` OVERHEAD:   265.48 ns
//!     `tracing-stackdriver` THROUGHPUT: 2.1041 µs
//!     `infinite-tracing` OVERHEAD:      29.647 ns
//!     `infinite-tracing` THROUGHPUT:    301.70 ns
//!     
//!     ==> The OVERHEAD relation is    265.48 / 29.647 = 8.96
//!     ==> The THROUGHPUT relation is  2104.1 / 301.70 = 6,97

use std::{hint::black_box, io::BufWriter};

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_tracers(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("Tracers");

    // baseline
    ///////////

    fn baseline_method(param: u32) -> Result<u32, u32> {
        Ok(param)
    }

    let bench_id = "baseline method call";
    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| black_box({ baseline_method(bench_id.as_ptr() as u32) }))
    });

    // infinite_tracing
    ///////////////////

    #[infinite_tracing::instrument(err)]
    fn infinite_tracing_overhead_method(param: u32) -> Result<u32, u32> {
        Ok(param)
    }

    #[infinite_tracing::instrument(err)]
    fn infinite_tracing_throughput_method(param: u32) -> Result<u32, u32> {
        Err(param)
    }

    // setup
    infinite_tracing::setup_infinite_tracing(BufWriter::with_capacity(8192, std::io::stderr()));

    // scenario: no tracing nor logging is done -- the method doesn't end in `Err`
    let bench_id = "`infinite-tracing` OVERHEAD";
    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| black_box({ infinite_tracing_overhead_method(bench_id.as_ptr() as u32) }))
    });

    // scenario: tracing & loggig is done -- the method ends in `Err`
    let bench_id = "`infinite-tracing` THROUGHPUT";
    group.bench_function(bench_id, |bencher| {
        bencher.iter(|| black_box({ infinite_tracing_throughput_method(bench_id.as_ptr() as u32) }))
    });

    // Cloudwalk's tracing-stackdriver
    //////////////////////////////////

    #[tracing::instrument]
    fn tracing_stackdriver_overhead_method(param: u32) -> Result<u32, u32> {
        Ok(param)
    }

    #[tracing::instrument(err)]
    fn tracing_stackdriver_throughput_method(param: u32) -> Result<u32, u32> {
        Err(param)
    }

    // setup
    use tracing_subscriber::layer::SubscriberExt;
    let stackdriver = tracing_stackdriver::layer(); // writes to std::io::Stdout
    let subscriber = tracing_subscriber::Registry::default().with(stackdriver);
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // scenario: no tracing nor logging is done -- the method doesn't end in `Err`
    let bench_id = "Cloudwalk's `tracing-stackdriver` OVERHEAD";
    group.bench_function(bench_id, |bencher| {
        bencher
            .iter(|| black_box({ tracing_stackdriver_overhead_method(bench_id.as_ptr() as u32) }))
    });

    // scenario: tracing & loggig is done -- the method ends in `Err`
    let bench_id = "Cloudwalk's `tracing-stackdriver` THROUGHPUT";
    group.bench_function(bench_id, |bencher| {
        bencher
            .iter(|| black_box({ tracing_stackdriver_throughput_method(bench_id.as_ptr() as u32) }))
    });

    group.finish();
}

criterion_group!(benches, bench_tracers);
criterion_main!(benches);

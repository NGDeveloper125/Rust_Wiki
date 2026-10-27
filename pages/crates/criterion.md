---
title: "criterion"
version: "0.8.2"
publisher: "Berkus Decker (berkus), Brook Heisler (bheisler), David Himmelstrup (lemmih), maintainers"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-27"
summary: "The standard Rust benchmarking harness: runs a function many times, applies statistics to separate signal from noise, and tells you whether a change made things faster or you imagined it."
domain: "Testing & benchmarking"
categories: ["development-tools::profiling"]
repository: "https://github.com/criterion-rs/criterion.rs"
docs: "https://docs.rs/criterion/0.8.2/criterion/"
---

## Overview

Rust has a built-in benchmark harness, `#[bench]` with `test::Bencher`, and it
has been nightly-only for a decade with no sign of stabilising. `criterion` is
what stable Rust uses instead, and it does considerably more than the built-in
one ever did.

A benchmark is a function taking `&mut Criterion`:

```
use criterion::Criterion;
use std::hint::black_box;

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn bench_fib(c: &mut Criterion) {
    // iter runs the closure as many times as the statistics need.
    c.bench_function("fib 20", |b| b.iter(|| fib(black_box(20))));
}
```

**Wiring it up is a two-part ritual** that catches everyone once. The file goes
in `benches/`, ends with the two macros, and `Cargo.toml` must disable the
default harness — otherwise `cargo bench` runs libtest instead and your
benchmarks never execute:

`criterion_group!(benches, bench_fib);` then `criterion_main!(benches);` at the
bottom of `benches/my_bench.rs`, and in `Cargo.toml` a `[[bench]]` section with
`name = "my_bench"` and **`harness = false`**. Criterion goes in
`[dev-dependencies]`.

**What you are paying for is the statistics.** Criterion runs your function for
a warm-up period, then collects many samples, fits a line through them, and
reports a confidence interval rather than a single number. On a re-run it
compares against the previous result and tells you whether the difference is
significant or within noise. That last part is the real value: a wall-clock
measurement on a laptop with a browser open is noisy enough that eyeballing two
numbers is worthless, and criterion is explicit about when it cannot tell.

**`black_box` is not optional.** Without it the optimiser can see that your
result is unused and delete the computation you are trying to measure, leaving
you benchmarking an empty loop. Use `std::hint::black_box` — it is stable now,
and criterion's own re-export exists mainly for older code.

**The trade-offs against the alternatives.** [`divan`](divan.md) measures the
same thing with a lighter attribute-based API and runs faster, at the cost of a
smaller feature set and a much smaller user base. [`gungraun`](gungraun.md)
counts instructions under Valgrind instead of measuring time, which makes it
deterministic and therefore far better suited to CI — where criterion's wall
clock is at the mercy of whatever else the runner is doing. Criterion remains
the default because it is the one everybody knows, it handles async and
parameterised benchmarks well, and its HTML reports with plots are genuinely
useful when you are exploring rather than regression-testing.

Note the **MSRV is 1.86**, which is recent for a dev-dependency, and that
`html_reports` is *not* a default feature in 0.8 — enable it if you want the
plots. Defaults are `rayon`, `plotters` and `cargo_bench_support`.

## When to use it

### Use case: Finding out whether an optimisation actually helped

The core loop: measure, change, measure again.

```
use criterion::Criterion;
use std::hint::black_box;

fn sum_loop(values: &[u64]) -> u64 {
    let mut total = 0;
    for &v in values {
        total += v;
    }
    total
}

fn sum_iter(values: &[u64]) -> u64 {
    values.iter().sum()
}

fn compare(c: &mut Criterion) {
    let data: Vec<u64> = (0..1000).collect();

    // Two named benchmarks, so each is tracked separately across runs.
    c.bench_function("sum_loop", |b| b.iter(|| sum_loop(black_box(&data))));
    c.bench_function("sum_iter", |b| b.iter(|| sum_iter(black_box(&data))));
}
```

**Why it fits:** criterion stores each benchmark's results under its name in
`target/criterion`, so the second `cargo bench` prints a comparison against the
first — "change: -14.2% (p = 0.00 < 0.05) Performance has improved". That
automatic before-and-after is the feature, and it is why benchmark names should
be stable: rename one and you lose its history. The `black_box` around `&data`
stops the optimiser from constant-folding a slice it can see the contents of.

### Use case: Measuring how cost scales with input size

Parameterised benchmarks, which produce a curve rather than a point.

```
use criterion::{BenchmarkId, Criterion, Throughput};

fn checksum(data: &[u8]) -> u64 {
    data.iter().map(|&b| b as u64).sum()
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");

    for size in [64usize, 1024, 16384] {
        let data = vec![7u8; size];

        // Throughput turns the result into bytes/second as well as time.
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| checksum(data))
        });
    }

    group.finish();
}
```

**Why it fits:** one timing tells you how fast something is; a series tells you
whether it is linear, and that is usually the question that matters. `Throughput`
is what converts "412 µs" into "39.7 GiB/s", which is the number you can compare
against memory bandwidth or a competing implementation. The group also gets a
line plot in the HTML report, where a complexity change is visible at a glance.

### Use case: Benchmarking something that needs fresh input each time

`iter_batched`, for when the operation destroys or mutates its input.

```
use criterion::{BatchSize, Criterion};

fn bench_sort(c: &mut Criterion) {
    c.bench_function("sort 1000 reversed", |b| {
        b.iter_batched(
            // Setup: not timed.
            || (0..1000u32).rev().collect::<Vec<_>>(),
            // Routine: timed.
            |mut v| {
                v.sort();
                v
            },
            BatchSize::SmallInput,
        )
    });
}
```

**Why it fits:** a plain `iter` would sort the same vector repeatedly, and after
the first iteration it is already sorted — so you would be measuring
best-case `sort` on sorted input, which is not what you meant. `iter_batched`
builds a fresh input per iteration and excludes that construction from the
timing. `BatchSize` controls how many are built at once: `SmallInput` for cheap
setup, `LargeInput` when the input is big enough that holding many would
disturb the cache, `PerIteration` when it must be exactly one.

## API map

The surface is small: a `Criterion` handle, benchmark groups, and a `Bencher`
with several timing strategies.

### Defining benchmarks

#### `bench_function`

One benchmark, one name.

```
use criterion::Criterion;
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    c.bench_function("parse u32", |b| {
        b.iter(|| "12345".parse::<u32>().unwrap())
    });

    // The closure receives a Bencher; everything inside iter is timed.
    c.bench_function("allocate", |b| {
        b.iter(|| black_box(vec![0u8; 1024]))
    });
}
```

**When to use it:** for any single measurement. The name is the identity used
for historical comparison and for the report, so make it descriptive and leave
it alone once it exists. Everything outside `iter` runs once, so hoist setup
there when it does not need repeating — and use `iter_batched` when it does.

#### `benchmark_group` and `BenchmarkId`

Several related measurements under one heading.

```
use criterion::{BenchmarkId, Criterion};

fn bench_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    for size in [10u64, 100, 1000] {
        group.bench_with_input(BenchmarkId::new("linear", size), &size, |b, &n| {
            b.iter(|| (0..n).find(|&x| x == n - 1))
        });
        group.bench_with_input(BenchmarkId::new("direct", size), &size, |b, &n| {
            b.iter(|| n - 1)
        });
    }

    // finish() is required: it writes the group's summary.
    group.finish();
}
```

**When to use it:** whenever benchmarks belong together — the same operation at
different sizes, or competing implementations of one thing. `BenchmarkId::new`
takes a function name and a parameter; `from_parameter` is the short form when
there is only one function. Forgetting `finish()` is a common slip and costs you
the group's comparison plots.

#### `Throughput`

Reporting rate instead of duration.

```
use criterion::{Criterion, Throughput};

fn bench_throughput(c: &mut Criterion) {
    let data = vec![0u8; 8192];
    let mut group = c.benchmark_group("hash");

    // Bytes gives MiB/s; Elements gives items/s.
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("sum", |b| {
        b.iter(|| data.iter().map(|&x| x as u64).sum::<u64>())
    });

    group.finish();
}
```

**When to use it:** for anything processing a quantity of input — parsing,
hashing, compression, serialisation. A rate is comparable across input sizes
where a duration is not, so it is the only way to see that your parser gets
slower per byte on large inputs. `Elements` is the right variant when the unit
is records rather than bytes.

### Timing strategies

#### `iter` and `iter_batched`

The two you will use.

```
use criterion::{BatchSize, Criterion};

fn bench(c: &mut Criterion) {
    // iter: the closure is pure, or at least repeatable.
    c.bench_function("pure", |b| b.iter(|| (0..100u64).sum::<u64>()));

    // iter_batched: fresh input per iteration, setup untimed.
    c.bench_function("consuming", |b| {
        b.iter_batched(
            || String::from("12345"),
            |s| s.into_bytes(),
            BatchSize::SmallInput,
        )
    });
}
```

**When to use it:** `iter` unless the routine consumes or mutates its input, in
which case `iter_batched`. There is also `iter_batched_ref` for a routine taking
`&mut T`, which avoids moving the input in and out. The distinction matters more
than it looks: benchmarking `sort` with `iter` silently measures the sorted case
after the first pass.

#### `iter_custom`

Timing you measure yourself.

```
use criterion::Criterion;
use std::hint::black_box;
use std::time::Instant;

fn bench_custom(c: &mut Criterion) {
    c.bench_function("manual timing", |b| {
        b.iter_custom(|iterations| {
            // Criterion asks for n iterations and takes your Duration.
            let start = Instant::now();
            for _ in 0..iterations {
                black_box((0..50u64).sum::<u64>());
            }
            start.elapsed()
        })
    });
}
```

**When to use it:** when criterion's own timing is the wrong instrument — GPU
work where you must read a device timer, an operation whose cost you want to
measure excluding a lock acquisition, or anything where the clock has to start
and stop inside the routine. You are responsible for returning the total elapsed
time for exactly the requested iteration count, and getting that wrong silently
scales every number.

### Configuration

#### Sample count and measurement time

Trading accuracy for patience.

```
use criterion::Criterion;
use std::time::Duration;

fn bench_configured(c: &mut Criterion) {
    let mut group = c.benchmark_group("slow operation");

    group
        .sample_size(20)                              // default is 100
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(1))
        .confidence_level(0.99)
        .noise_threshold(0.05);

    group.bench_function("work", |b| b.iter(|| (0..1000u64).sum::<u64>()));
    group.finish();
}
```

**When to use it:** `sample_size` is the one you will reach for, because the
default of 100 samples makes a slow benchmark take minutes and criterion will
warn you about it. The minimum is 10. Raise `measurement_time` instead when the
operation is fast but noisy. `noise_threshold` sets how large a change must be
before it is reported as a change at all — worth raising on a shared machine to
stop every run claiming a regression.

#### `black_box`

Stopping the optimiser from deleting your benchmark.

```
use criterion::Criterion;
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    // Without black_box the compiler may compute this at compile time
    // and leave you timing an empty loop.
    c.bench_function("with", |b| b.iter(|| black_box(2u64) + black_box(2)));

    // Also wrap inputs the optimiser could otherwise see through.
    let data = vec![1u8, 2, 3];
    c.bench_function("input", |b| {
        b.iter(|| black_box(&data).len())
    });
}
```

**When to use it:** around anything the compiler could constant-fold, and around
results that are otherwise unused. `std::hint::black_box` is the stable one and
what you should import; criterion's re-export predates stabilisation. It is not
a perfect optimisation barrier — nothing in safe Rust is — but a benchmark
reporting a suspicious sub-nanosecond figure is almost always missing one.

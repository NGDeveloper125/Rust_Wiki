---
title: "Benchmarking"
area: "Testing & Tooling"
embedded_support: partial
groups: ["Testing & Tooling", "Testing & Documenting Code", "Testing"]
related_syntax: []
see_also: ["Unit tests", "Doc tests"]
---

## Explanation

Benchmarking measures how fast code runs rather than whether it produces
the right answer — a different question from what a
[unit test](unit-tests.md) answers, and one that needs a different kind
of tool. A single timed run is noisy: OS scheduling, CPU frequency
scaling, and cache state all shift the number from one run to the next,
so a real benchmark runs the code many times and reports a statistic
(typically a mean and a spread) rather than one raw duration.

Rust's standard library ships an unstable `#[bench]` attribute and a
harness for it, but it sits behind `#![feature(test)]` and the internal
`test` crate — both nightly-only, unavailable on stable Rust. In
practice, the ecosystem standard for real benchmarking is the third-party
`criterion` crate: it runs on stable, takes care of warm-up iterations and
statistical outlier rejection, and can compare a new run against a saved
baseline to flag regressions. `criterion` isn't among the small set of
crates this wiki shows in compiling code (see the crate policy this site
follows), so it's described here in prose rather than in a runnable
snippet; the compiling example below instead uses `std::time::Instant`
directly, which is honest but cruder — a single measured duration, with
none of `criterion`'s warm-up or statistical rigor.

Because of that gap, a `std::time::Instant`-based timing is best treated
as a rough gut-check ("is this obviously faster than that?"), not as a
substitute for a proper benchmark harness when a decision actually rides
on the number.

## Basic usage example

```
use std::time::Instant;

fn sum_of_squares(n: u64) -> u64 {
    (1..=n).map(|x| x * x).sum()
}

let start = Instant::now(); // <- starts the measurement
let total = sum_of_squares(1_000_000);
let elapsed = start.elapsed(); // <- duration since `start`
println!("total={total}, took {elapsed:?}");
```

## Best practices & deeper information

### Scenario: Numeric computation

Comparing two candidate implementations of a hot numeric loop — here, an
iterator chain against a manual accumulator — with simple timing is
enough to sanity-check that a rewrite is actually faster before reaching
for a full benchmark harness.

```
use std::time::Instant;

fn sum_iterator(values: &[f64]) -> f64 {
    values.iter().sum() // <- candidate A
}

fn sum_manual_loop(values: &[f64]) -> f64 {
    let mut total = 0.0;
    for &v in values { // <- candidate B
        total += v;
    }
    total
}

let readings: Vec<f64> = (0..1_000_000).map(|i| i as f64 * 0.5).collect();

let start = Instant::now();
let a = sum_iterator(&readings);
println!("iterator: {:?}", start.elapsed()); // <- rough, single-sample timing

let start = Instant::now();
let b = sum_manual_loop(&readings);
println!("manual loop: {:?}", start.elapsed());

assert_eq!(a, b);
```

**Why this way:** the [Rust standard library docs](https://doc.rust-lang.org/std/time/struct.Instant.html)
document `Instant` as monotonic and suitable for measuring elapsed time,
but a single sample like this is only a starting signal — for a change
that genuinely needs statistical confidence (a library-level performance
claim, a regression gate in CI), `criterion`'s repeated-sampling approach
is the better tool, per its own
[documentation](https://bheisler.github.io/criterion.rs/book/).

### Scenario: Testing

Cargo has a first-class place for benchmarks — a `benches/` directory,
parallel to `tests/` — but what harness runs there differs sharply
between nightly's built-in `#[bench]` and the stable-friendly `criterion`
crate, and being upfront about that split matters more here than in most
other scenarios.

```
// benches/sum_bench.rs — nightly only; requires `#![feature(test)]` and the
// unstable `test` crate. Shown for reference: this does NOT compile on stable.
#![feature(test)]
extern crate test;
use test::Bencher;
use my_crate::sum_of_squares;

#[bench] // <- the unstable, nightly-only benchmark harness
fn bench_sum_of_squares(b: &mut Bencher) {
    b.iter(|| sum_of_squares(1_000));
}
```

**Why this way:** because `#[bench]` never stabilized, real projects
benchmark on stable with `criterion` instead — a `#[criterion::criterion]`-
annotated function in `benches/`, run via `cargo bench`, gets warm-up
iterations and a statistical report without needing nightly at all; the
[Rust standard library docs](https://doc.rust-lang.org/std/index.html)
still list the `test` crate as unstable, which is the direct reason the
ecosystem standardized on a third-party crate for this instead.

## Embedded Rust Notes

**Partial support.** Both benchmarking paths described above assume a
host environment: the unstable `#[bench]` harness needs nightly `std`,
and `criterion` needs `std` plus the ability to print a statistical
report, neither of which exists on bare metal. On real embedded targets,
"benchmarking" usually means something more direct — reading a hardware
cycle counter (such as Cortex-M's `DWT` cycle counter) immediately before
and after the code of interest, rather than running `cargo bench` at all.
This measures actual on-target execution time, including effects like
flash wait states, that a host-side benchmark could never capture.

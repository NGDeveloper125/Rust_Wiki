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

## Explanation (Embedded)

Both host benchmarking paths above measure the wrong CPU entirely for
embedded work: `#[bench]` and `criterion` time how fast code runs on the
*development machine*, which has a different instruction set, clock
speed, cache hierarchy, and memory system than a microcontroller. A
number from `cargo bench` says nothing about how long a routine takes on
the actual target — it isn't a rough approximation that's merely less
precise, it's measuring a genuinely different piece of silicon, so it
isn't meaningful for target-hardware timing at all.

Real embedded timing measurement instead reads the target's own notion
of elapsed time. The most common approach uses the Cortex-M `DWT` (Data
Watchpoint and Trace) unit's cycle counter, accessed via the `cortex-m`
crate: enable it once, read the cycle count immediately before and after
the code of interest, and the difference is an exact, on-target cycle
count — including real effects a host benchmark could never see, like
flash wait states or bus contention. Where even more ground truth is
needed — end-to-end latency including interrupt response time, or
comparing against an external event — the classic technique is toggling
a GPIO pin high just before the code runs and low just after, then
measuring the pulse width with a logic analyzer or oscilloscope
externally. Neither technique gives `criterion`'s statistical rigor out
of the box, but both measure the thing that actually matters for
firmware: real time on the real chip.

## Basic usage example (Embedded)

```
use cortex_m::peripheral::DWT;

fn sum_of_squares(n: u32) -> u32 {
    (1..=n).map(|x| x * x).sum()
}

fn measure(dwt: &mut DWT) {
    let start = dwt.cyccnt.read(); // <- on-target cycle count before
    let total = sum_of_squares(1_000);
    let elapsed_cycles = dwt.cyccnt.read().wrapping_sub(start); // <- exact cycles spent on the target CPU
    defmt::info!("total={}, took {} cycles", total, elapsed_cycles);
}
```

## Best practices & deeper information (Embedded)

### Scenario: Numeric computation

Comparing two candidate implementations of a hot numeric routine only
means something for a firmware's actual performance budget if the timing
comes from the target chip itself, not the development host.

```
use cortex_m::peripheral::DWT;

fn sum_iterator(values: &[i32]) -> i32 {
    values.iter().sum() // <- candidate A
}

fn sum_manual_loop(values: &[i32]) -> i32 {
    let mut total = 0;
    for &v in values { // <- candidate B
        total += v;
    }
    total
}

fn compare(dwt: &mut DWT, readings: &[i32]) {
    let start = dwt.cyccnt.read();
    let a = sum_iterator(readings);
    let cycles_a = dwt.cyccnt.read().wrapping_sub(start); // <- on-target cycle count, candidate A

    let start = dwt.cyccnt.read();
    let b = sum_manual_loop(readings);
    let cycles_b = dwt.cyccnt.read().wrapping_sub(start); // <- on-target cycle count, candidate B

    defmt::assert_eq!(a, b);
    defmt::info!("iterator: {} cycles, manual loop: {} cycles", cycles_a, cycles_b);
}
```

**Why this way:** the two candidates can rank differently on the target's
Cortex-M core than on a desktop CPU — different pipeline depth, no
speculative execution, flash wait states on every fetch — so a decision
that rides on which is actually faster in the shipping firmware has to be
measured with the `DWT` cycle counter on real hardware, not with
`std::time::Instant` on the host.

### Scenario: Testing

A routine's execution time sometimes needs to be verified against an
external, ground-truth clock — end-to-end latency from an interrupt
firing to a response being ready, say — which a cycle counter inside the
same chip can't independently confirm.

```
use embedded_hal::digital::OutputPin;

fn handle_sensor_interrupt(trigger_pin: &mut impl OutputPin, adc_sample: impl FnOnce() -> u16) -> u16 {
    trigger_pin.set_high().ok(); // <- pulse starts: visible on a logic analyzer/oscilloscope
    let sample = adc_sample();
    trigger_pin.set_low().ok(); // <- pulse ends: the width between edges is the measured duration
    sample
}
```

**Why this way:** toggling a spare GPIO pin around the code under test
gives an external instrument a hardware-level timestamp pair that's
independent of the CPU whose timing is being measured, which is the
standard way to validate interrupt-to-response latency or confirm a
`DWT`-based measurement against ground truth when the stakes (a real-time
deadline) justify the extra test rig.

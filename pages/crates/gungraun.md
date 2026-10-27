---
title: "gungraun"
version: "0.20.0"
publisher: "Joining7943, gamma0987"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-27"
summary: "Benchmarks by counting instructions under Valgrind rather than measuring time, so results are deterministic and usable in CI. Formerly published as `iai-callgrind`."
domain: "Testing & benchmarking"
categories: ["development-tools::testing", "development-tools::profiling"]
repository: "https://github.com/gungraun/gungraun"
docs: "https://docs.rs/gungraun/0.20.0/gungraun/"
---

## Overview

Every wall-clock benchmark has the same weakness: the number depends on what
else the machine was doing. `criterion` and `divan` fight this with statistics,
and on a quiet workstation they win. On a shared CI runner they do not, which
is why performance regressions are so rarely caught automatically.

`gungraun` takes the other route. It runs your code under Valgrind's Callgrind
and **counts instructions, cache accesses and memory writes** instead of timing
anything. Those counts are deterministic: the same binary on the same input
produces the same numbers on a loaded machine, a quiet one, and a colleague's
laptop. A diff of one instruction is a real diff.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

#[library_benchmark]
fn bench_fib() -> u64 {
    black_box(fib(10))
}

// A bench file is these three pieces: the benchmarks, a group, and main!.
library_benchmark_group!(name = fibonacci; benchmarks = bench_fib);
main!(library_benchmark_groups = fibonacci);
```

**It was called `iai-callgrind` until recently.** The crate and its repository
were renamed to `gungraun`; `iai-callgrind` stopped at 0.16.1 in July 2025 and
development continues here. If you are following a tutorial or a `Cargo.toml`
mentioning `iai-callgrind`, this is the same project under a new name. The
original `iai`, which both descend from, has been unmaintained for years.

**The hard requirement is Valgrind**, and it shapes where this is usable.
Valgrind runs on Linux and other Unix systems; there is no usable Windows
support, and macOS support lags. In practice this means Linux CI, which is also
exactly where it is most valuable. It must be installed on the machine running
the benchmarks — the crate shells out to it.

**What the numbers are and are not.** An instruction count is not a time.
Fewer instructions usually means faster, but not always: a branch-free version
doing more work can beat a branchy one doing less, and cache behaviour often
dominates both. Gungraun reports cache and branch statistics alongside the
instruction count and estimates cycles from them, which helps, but the estimate
is a model rather than a measurement. **Use it to detect change, not to
establish absolute speed** — and when it flags a regression, confirm the wall
clock with [`criterion`](criterion.md) before acting.

The other cost is that running under Valgrind is roughly fifty times slower
than running natively, so a benchmark doing real work needs a small input.
Against that, it needs only one iteration rather than hundreds of samples, so
the totals often come out comparable.

Requires **Rust 1.88 and edition 2024**, which is aggressive. Benchmarks go in
`benches/` with `harness = false`, as with the other two.

## When to use it

### Use case: Catching performance regressions in CI

The case that justifies the Valgrind dependency.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn encode(input: &[u8]) -> Vec<u8> {
    input.iter().map(|b| b.wrapping_add(1)).collect()
}

// The count for this is identical on every machine and every run,
// so a change in it is a change in the code, not in the weather.
#[library_benchmark]
fn bench_encode() -> Vec<u8> {
    let data = vec![7u8; 256];
    black_box(encode(black_box(&data)))
}

library_benchmark_group!(name = encoding; benchmarks = bench_encode);
main!(library_benchmark_groups = encoding);
```

**Why it fits:** a CI job comparing wall-clock timings against a stored baseline
produces false alarms often enough that teams stop reading it. Instruction
counts do not drift, so the threshold can be tight — a few per cent, or even
zero — and an alert means something. This is the one benchmarking approach that
works properly on a noisy shared runner, and the reason to accept that it only
runs on Linux.

### Use case: Comparing implementations at fixed inputs

Parameterised benchmarks, named per case.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

// Each #[bench::name(args)] is a separate, separately-reported benchmark.
#[library_benchmark]
#[bench::small(5)]
#[bench::medium(10)]
#[bench::large(15)]
fn bench_fib(n: u64) -> u64 {
    black_box(fib(black_box(n)))
}

library_benchmark_group!(name = fibonacci; benchmarks = bench_fib);
main!(library_benchmark_groups = fibonacci);
```

**Why it fits:** the attribute names each case, so the report lists `small`,
`medium` and `large` separately and each gets its own history. Because the
measurement is exact, the ratio between them is meaningful in a way noisy
timings are not — if `large` costs 11 times `medium` today and 14 times
tomorrow, something changed in the algorithm rather than in the machine.

### Use case: Keeping setup out of the count

`setup` functions, which run outside the measured region.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn make_data() -> Vec<u32> {
    (0..1000).rev().collect()
}

// setup runs before measurement begins; its instructions are not counted.
#[library_benchmark]
#[bench::reversed(setup = make_data)]
fn bench_sort(mut data: Vec<u32>) -> Vec<u32> {
    data.sort();
    black_box(data)
}

library_benchmark_group!(name = sorting; benchmarks = bench_sort);
main!(library_benchmark_groups = sorting);
```

**Why it fits:** building the input is work, and under instruction counting that
work is just as visible as the thing you meant to measure — more so, since
allocating a thousand-element vector is not cheap. `setup` is gungraun's
equivalent of criterion's `iter_batched`, and it matters more here, because
there is no statistical averaging to hide the difference. There is a matching
`teardown` for cleanup that should likewise not be counted.

## API map

Three macros and a configuration type. The surface is small; the complexity is
in Valgrind, not in the API.

### Library benchmarks

#### `#[library_benchmark]`

Marks a function as a benchmark.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

// The return value is kept so the work is not optimised away.
#[library_benchmark]
fn bench_sum() -> u64 {
    black_box((0..100u64).sum())
}

library_benchmark_group!(name = arithmetic; benchmarks = bench_sum);
main!(library_benchmark_groups = arithmetic);
```

**When to use it:** on every benchmark of a function in your own crate — the
"library" in the name distinguishes it from benchmarking a whole binary.
There is no per-benchmark rename parameter; the reported name is the
function's. `black_box` matters as much as in the other harnesses, and
arguably more: with
only one iteration there is no noise to disguise a body the optimiser removed,
so a suspiciously small count is the symptom.

#### `#[bench::name(...)]` and `setup`

Cases and their inputs.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn build(n: usize) -> String {
    "x".repeat(n)
}

#[library_benchmark]
#[bench::short(args = (4,), setup = build)]
#[bench::long(args = (64,), setup = build)]
fn bench_len(s: String) -> usize {
    black_box(s.len())
}

library_benchmark_group!(name = lengths; benchmarks = bench_len);
main!(library_benchmark_groups = lengths);
```

**When to use it:** for every variant you want reported separately. The `args`
tuple is passed to `setup`, whose return value becomes the benchmark function's
argument — so the construction cost stays outside the count while the parameter
stays visible in the report. Without `setup`, `args` goes straight to the
benchmark function, which is right when the argument is a cheap scalar.

#### `library_benchmark_group!` and `main!`

Registration.

```
use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
fn bench_one() -> u64 {
    black_box(1u64 + 1)
}

#[library_benchmark]
fn bench_two() -> u64 {
    black_box(2u64 * 2)
}

// Groups collect benchmarks; main! collects groups.
library_benchmark_group!(
    name = arithmetic;
    benchmarks = bench_one, bench_two
);

main!(library_benchmark_groups = arithmetic);
```

**When to use it:** once per file. The group gives the benchmarks a shared name
in the report and a place to hang shared configuration; `main!` is what makes
the file an executable benchmark target, and it must list every group. The
semicolon separators inside these macros are easy to get wrong — they are
semicolons, not commas, between the named sections.

### Configuration

#### Valgrind tools and options

Choosing what to measure.

```
// no-compile — illustrative configuration, see the gungraun guide
use gungraun::{LibraryBenchmarkConfig, Callgrind, EventKind};

let config = LibraryBenchmarkConfig::default()
    .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)]));

// Applied at the group or main level:
//   library_benchmark_group!(
//       name = grp;
//       config = config;
//       benchmarks = bench_one
//   );
```

**When to use it:** when the defaults are not what you want to measure.
Callgrind is the default tool and gives instruction and cache counts; the crate
also drives DHAT for heap profiling, Massif, Memcheck and Linux `perf`, which is
the reason for the rename away from a Callgrind-specific name. Soft and hard
limits are how a benchmark fails CI on a regression rather than merely reporting
one — a hard limit turns a threshold breach into a non-zero exit.

#### Binary benchmarks

Measuring a whole executable rather than a function.

```
// no-compile — requires a binary target in the same crate
use gungraun::{binary_benchmark, Command};

#[binary_benchmark]
#[bench::help(args = ["--help"])]
fn bench_cli(arg: &str) -> Command {
    Command::new(env!("CARGO_BIN_EXE_my-tool"))
        .arg(arg)
        .build()
}

// Registered with binary_benchmark_group! and listed in main!.
```

**When to use it:** for a command-line tool, where the interesting cost includes
start-up, argument parsing and I/O rather than one function. It measures the
whole process under Valgrind, so the count covers everything the binary does —
which makes it the right instrument for "did my CLI get slower to start", a
question function-level benchmarks cannot answer. `CARGO_BIN_EXE_<name>` is the
Cargo-provided path to your built binary.

---
title: "divan"
version: "0.1.21"
publisher: "Nikolai Vazquez (nvzqz), Thom Chiovoloni (thomcc)"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-27"
summary: "A benchmarking harness built around one attribute: put `#[divan::bench]` on a function and it is a benchmark. Covers generics and parameters that `criterion` needs a builder for, and runs faster."
domain: "Testing & benchmarking"
categories: ["development-tools::profiling"]
repository: "https://github.com/nvzqz/divan"
docs: "https://docs.rs/divan/0.1.21/divan/"
---

## Overview

`divan` benchmarks a function by putting an attribute on it. That is the whole
design, and it is a deliberate answer to [`criterion`](criterion.md), where a
benchmark is a closure registered inside a function registered in a macro:

```
use divan::black_box;

fn fib(n: u64) -> u64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

// This is a complete benchmark.
#[divan::bench]
fn fib_20() -> u64 {
    fib(black_box(20))
}
```

The returned value is kept from the optimiser automatically, so there is no
`black_box` needed on the way out — only on inputs the compiler could otherwise
fold. The file still goes in `benches/`, still needs `harness = false` in
`Cargo.toml`, and needs `fn main() { divan::main(); }` at the bottom, which
collects every attributed function in the file.

**Where it is clearly better than criterion** is anything combinatorial.
Parameters and generic type arguments are attribute arguments, and divan runs
the cross product:

```
// Three benchmarks from one function.
#[divan::bench(args = [16, 64, 256])]
fn sum_to(n: u64) -> u64 {
    (0..divan::black_box(n)).sum()
}

// One benchmark per type.
#[divan::bench(types = [u32, u64, i128])]
fn default_of<T: Default>() -> T {
    T::default()
}
```

Doing that in criterion means a loop, a `BenchmarkId` and a group; doing the
generic version means a macro. Divan also reports allocation counts alongside
timings, which criterion does not, and it runs noticeably faster because it
spends less time collecting samples.

**Where criterion is still the safer choice.** Divan is version 0.1 and much
less widely used, so there is less written about it and fewer people to ask. It
has no built-in HTML reports or plots — output is a table in the terminal. And
it does not compare against previous runs: criterion's "change: -14.2%
(p = 0.00)" against stored history is its single most useful feature for
regression work, and divan has no equivalent. Divan tells you what the code
costs now; criterion tells you whether you made it worse.

So: divan when you are exploring a design space, benchmarking across types and
sizes, or want quick numbers. Criterion when you are tracking performance over
time. [`gungraun`](gungraun.md) when you need results that do not move between
CI runs, which neither of these can offer, since both measure wall-clock time.

Last released in April 2025 and still committed to, with no required
dependencies to speak of and an MSRV of 1.80.

## When to use it

### Use case: Benchmarking the same operation across several types

The case that is awkward everywhere else.

```
use divan::black_box;
use std::collections::{BTreeSet, HashSet};

// One benchmark per type, from one function body.
#[divan::bench(types = [HashSet<u32>, BTreeSet<u32>])]
fn insert_1000<S>() -> S
where
    S: Default + Extend<u32>,
{
    let mut set = S::default();
    set.extend(black_box(0..1000u32));
    set
}
```

**Why it fits:** comparing two container types is one of the commonest reasons
to benchmark at all, and expressing it generically means the body cannot
accidentally differ between the two arms. The equivalent in criterion is either
duplicated code or a declarative macro generating the registrations, both of
which obscure what is being measured. Divan prints the results as a tree grouped
by the function, so the comparison is adjacent in the output.

### Use case: Measuring a scaling curve without a builder

Parameterised benchmarks as attribute arguments.

```
use divan::black_box;

#[divan::bench(args = [8usize, 64, 512, 4096])]
fn checksum(size: usize) -> u64 {
    let data = vec![7u8; size];
    black_box(&data).iter().map(|&b| b as u64).sum()
}

// consts works the same way for const-generic parameters.
#[divan::bench(consts = [4, 16])]
fn fixed_array<const N: usize>() -> usize {
    black_box([0u8; N]).len()
}
```

**Why it fits:** the parameters are visible in the signature, so the benchmark
reads as the function it is. Note the flaw in the first example, which is
common and worth naming: the `vec!` allocation happens *inside* the timed body,
so you are measuring allocation plus checksum. The next section is how to fix
that, and divan makes the fix shorter than criterion does.

### Use case: Excluding setup from the measurement

`Bencher` with `with_inputs`, for the destructive case.

```
use divan::Bencher;

#[divan::bench]
fn sort_reversed(bencher: Bencher) {
    bencher
        // Built fresh per iteration, and not timed.
        .with_inputs(|| (0..1000u32).rev().collect::<Vec<_>>())
        // Timed. bench_local_values takes each input by value.
        .bench_local_values(|mut v| {
            v.sort();
            v
        });
}

#[divan::bench]
fn search(bencher: Bencher) {
    let haystack: Vec<u32> = (0..10_000).collect();
    bencher
        .with_inputs(|| 9_999u32)
        // bench_refs passes &mut to the routine instead.
        .bench_values(|needle| haystack.binary_search(&needle));
}
```

**Why it fits:** sorting an already-sorted vector measures the wrong thing, and
`with_inputs` is the fix — it is divan's `iter_batched`, with the batching
strategy chosen for you rather than specified. Taking a `Bencher` parameter is
how a benchmark opts into the builder style; a function without one is timed
whole. `bench_local_values` is for non-`Send` work on one thread, and
`bench_values` for the general case.

## API map

One attribute, one `Bencher`, and a handful of counters. There is very little
to learn, which is the point.

### The attribute

#### `#[divan::bench]`

The whole registration mechanism.

```
use divan::black_box;

// A benchmark is a function. The return value is kept from the optimiser.
#[divan::bench]
fn parse() -> u32 {
    black_box("12345").parse().unwrap()
}

// Named, when the function name is not the right label.
#[divan::bench(name = "parse a large number")]
fn parse_big() -> u64 {
    black_box("18446744073709551615").parse().unwrap()
}
```

**When to use it:** on every benchmark. The function may take no arguments and
return anything; returning the result is how you keep it alive, and divan drops
it outside the timed region so destructors do not pollute the measurement — a
detail criterion leaves to you. `fn main() { divan::main(); }` must be present
in the file for any of it to run.

#### `args`, `types` and `consts`

Generating several benchmarks from one function.

```
use divan::black_box;

// Runtime values.
#[divan::bench(args = [1u32, 10, 100])]
fn multiply(n: u32) -> u32 {
    black_box(n) * 2
}

// Type parameters.
#[divan::bench(types = [i32, i64])]
fn zero<T: Default + std::fmt::Debug>() -> T {
    T::default()
}

// Const generic parameters.
#[divan::bench(consts = [8, 32])]
fn array_sum<const N: usize>() -> usize {
    black_box([1usize; N]).iter().sum()
}
```

**When to use it:** whenever you would otherwise write a loop registering
benchmarks. `args` accepts any expression producing an iterator of values, so a
constant slice or a range both work, and the values appear in the output as the
benchmark's parameter. The three can be combined, in which case divan runs the
full cross product — which is powerful and also how you accidentally start a
twenty-minute benchmark run.

#### Sample configuration

Controlling how long it takes.

```
use divan::black_box;

// sample_count is how many samples; sample_size is iterations per sample.
#[divan::bench(sample_count = 20, sample_size = 100)]
fn tuned() -> u64 {
    black_box(40u64) + 2
}

// Or bound by time rather than count.
#[divan::bench(max_time = 2)]
fn time_bounded() -> u64 {
    (0..black_box(1000u64)).sum()
}
```

**When to use it:** when the defaults make a slow benchmark unbearable, or a
fast one imprecise. The two-level structure is worth understanding —
`sample_size` iterations are timed together as one sample, which amortises the
clock's own cost over a cheap operation, and `sample_count` samples make up the
statistics. For an operation taking microseconds the defaults are fine; for one
taking seconds, lower both.

### Benchmarks with setup

#### `Bencher::with_inputs`

Per-iteration setup, excluded from timing.

```
use divan::Bencher;

#[divan::bench]
fn consume_string(bencher: Bencher) {
    bencher
        .with_inputs(|| String::from("12345"))
        .bench_local_values(|s| s.into_bytes());
}

#[divan::bench]
fn mutate_vec(bencher: Bencher) {
    bencher
        .with_inputs(|| vec![3u32, 1, 2])
        .bench_local_refs(|v| {
            v.sort();
            v.len()
        });
}
```

**When to use it:** whenever the routine consumes or mutates its input.
`bench_local_values` hands each input over by value, `bench_local_refs` passes
`&mut`, and the non-`local` variants are for work that must be `Send`. Divan
decides how many inputs to prepare at a time, which is the main simplification
over criterion's explicit `BatchSize`.

#### Counters

Reporting throughput alongside time.

```
use divan::counter::{BytesCount, ItemsCount};
use divan::{black_box, Bencher};

#[divan::bench]
fn hash_bytes(bencher: Bencher) {
    let data = vec![0u8; 4096];
    bencher
        // Adds a bytes-per-second column to the output.
        .counter(BytesCount::of_slice(&data))
        .bench(|| black_box(&data).iter().map(|&b| b as u64).sum::<u64>());
}

#[divan::bench]
fn process_items(bencher: Bencher) {
    bencher
        .counter(ItemsCount::new(100usize))
        .bench(|| black_box((0..100u64).sum::<u64>()));
}
```

**When to use it:** for anything measured per byte or per record, which is the
same reasoning as criterion's `Throughput`. `BytesCount::of_slice` reads the
length for you; `ItemsCount::new` takes a count you supply.
`divan::counter::CharsCount` exists for text. Divan prints these as extra
columns rather than replacing the timing, so you see both.

### Organising output

#### `#[divan::bench_group]`

Grouping benchmarks and sharing configuration.

```
// Settings on the group apply to every benchmark inside it.
#[divan::bench_group(sample_count = 10)]
mod parsing {
    use divan::black_box;

    #[divan::bench]
    fn small() -> u32 {
        black_box("42").parse().unwrap()
    }

    #[divan::bench]
    fn large() -> u64 {
        black_box("9999999999").parse().unwrap()
    }
}
```

**When to use it:** when a set of benchmarks shares configuration, or when the
output needs structure. Groups are modules, so the nesting is the module tree
and divan prints it as one — which scales better than criterion's flat names
once you have thirty benchmarks. Configuration on the group is inherited and can
be overridden per benchmark.

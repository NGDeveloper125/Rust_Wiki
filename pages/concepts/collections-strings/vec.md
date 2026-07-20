---
title: "Vec<T>"
area: "Collections & Strings"
embedded_support: partial
groups: ["Collections & Strings", "Working with Collections", "Collections"]
related_syntax: ["[ ]", ".", "!"]
see_also: ["Arrays vs Vec", "Slices", "HashMap & HashSet", "BTreeMap & BTreeSet"]
---

## Explanation

`Vec<T>` is Rust's growable array: a contiguous, heap-allocated buffer of
`T` that can gain and lose elements at runtime. Unlike a fixed-size array
(see [Arrays vs Vec](arrays-vs-vec.md) for that comparison), a `Vec`
tracks two numbers alongside its buffer pointer — a `len` (how many
elements are actually in use) and a `capacity` (how much backing storage
is currently allocated). Pushing an element only needs a fresh allocation
when `len` catches up to `capacity`; otherwise it just writes into
already-owned memory and increments `len`. That distinction between
length and capacity is the whole story of how `Vec` grows.

When a `Vec` does need to grow, it doesn't allocate one element at a
time — it reallocates to a larger capacity (`std`'s implementation
roughly doubles it) and moves the existing elements over. Because the
buffer grows geometrically rather than linearly, the *total* cost of
pushing `n` elements one at a time, summed over every reallocation,
stays proportional to `n` — this is what "amortized O(1) push" means:
any individual `push` might trigger a reallocation, but the cost
averaged over a long run of pushes is constant. Calling
`Vec::with_capacity(n)` up front — when the eventual size is known or
estimable — skips the repeated reallocate-and-copy cycle entirely.

A `Vec<T>` derefs to `&[T]`, so everything that works on a
[slice](slices.md) — indexing, iteration, `.len()`, `.iter()` — works on
a `Vec` without any extra ceremony; a `Vec` is best thought of as an
owning, growable slice plus the bookkeeping needed to grow it. That's
also why so much of the standard library — `String` for text (see
[String vs &str](string-vs-str.md)) being the closest analogy — follows
the same owned-growable-buffer-plus-borrowed-view shape.

`Vec<T>` is the default sequence type in Rust: reach for it first for any
homogeneous, ordered collection whose size isn't a fixed compile-time
fact, and only reach for something more specialized —
[`HashMap`/`HashSet`](hashmap-and-hashset.md) for key lookups,
[`BTreeMap`/`BTreeSet`](btreemap-and-btreeset.md) for ordered key
lookups, a fixed-size array for compile-time-known lengths — once the
problem actually calls for it.

## Basic usage example

```
let mut readings: Vec<f64> = Vec::new();
readings.push(21.5); // <- grows the Vec; len becomes 1, capacity may grow past 1
readings.push(22.0);

println!("{:?}", readings); // [21.5, 22.0]
```

**Restriction:** indexing past `len` (`readings[5]`) panics at runtime
rather than being caught at compile time, same as a slice — use
`.get(i)`, which returns `Option<&T>`, when the index isn't already
known to be in bounds.

## Best practices & deeper information

### Scenario: Creating a new object

When the eventual size of a `Vec` is known or can be estimated ahead of
time, pre-allocating with `with_capacity` avoids the repeated
reallocate-and-copy cycle that plain `push`-ing from an empty `Vec` would
otherwise pay for as it grows.

```
fn collect_batch(sensor_count: usize) -> Vec<f64> {
    let mut readings = Vec::with_capacity(sensor_count); // <- one allocation sized for the known count
    for id in 0..sensor_count {
        readings.push(read_sensor(id)); // <- no reallocation happens during this loop
    }
    readings
}

fn read_sensor(id: usize) -> f64 {
    10.0 + id as f64 // stand-in for an actual sensor read
}
```

**Why this way:** the [standard library docs](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation)
call out `with_capacity` specifically for this case — when the number of
elements is known up front, it turns several amortized reallocations
into exactly one.

### Scenario: Working with collections

Iterator chains built on `.iter()`/`.map()`/`.filter()` and collected
back into a `Vec` are the idiomatic way to transform one sequence into
another, and `.retain()` is the idiomatic way to remove elements in
place without rebuilding the whole `Vec` by hand.

```
let mut orders: Vec<u32> = vec![1250, 300, 4200, 75, 980]; // order totals, in cents... times 100 for readability

orders.retain(|&total| total >= 500); // <- keeps only orders that clear the minimum, shifting the rest down in place
let discounted: Vec<u32> = orders.iter().map(|total| total * 9 / 10).collect(); // <- built into a fresh Vec

println!("{:?}", discounted);
```

**Why this way:** `retain` mutates the existing buffer in place instead
of allocating a new `Vec` and filtering into it, which the
[std docs](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain)
recommend over a manual filter-and-rebuild whenever the result stays a
`Vec` of the same element type.

### Scenario: Designing a public API

A function that produces a sequence should usually return an owned
`Vec<T>` when the caller needs to keep, store, or further mutate the
result, while a function that only needs to *read* one should accept
`&[T]` rather than `&Vec<T>` so it also works for arrays and sub-slices.

```
fn top_scores(mut scores: Vec<u32>, n: usize) -> Vec<u32> { // <- returns an owned Vec: caller gets a value it can keep
    scores.sort_unstable_by(|a, b| b.cmp(a));
    scores.truncate(n);
    scores
}

fn average(scores: &[u32]) -> f64 { // <- &[u32] accepts this Vec, an array, or a slice of either
    scores.iter().sum::<u32>() as f64 / scores.len() as f64
}

let leaderboard = top_scores(vec![88, 42, 95, 71, 60], 3);
println!("{:.1}", average(&leaderboard));
```

**Why this way:** the API Guidelines'
[C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generic-types-c-generic)
advice is to accept the least specific type a function actually needs —
`&[T]` for reading — while returning the concrete owned `Vec<T>` when the
caller genuinely takes ownership of new data.

## Embedded Rust Notes

**Partial support.** `Vec<T>` lives in the `alloc` crate, not `core` — on
hosted `std` targets it just works, but under `#![no_std]` it requires
`extern crate alloc` plus a configured `#[global_allocator]` before
`alloc::vec::Vec` is usable at all. Where no heap is available, or where
allocation failure and fragmentation are unacceptable,
`heapless::Vec<T, N>` provides a fixed-*capacity* (chosen at compile
time), allocation-free alternative that still tracks a runtime length
and supports `push`/`pop`, at the cost of a hard upper bound `N`.

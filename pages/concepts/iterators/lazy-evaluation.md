---
title: "Lazy evaluation"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Functional Programming", "Iterating & Transforming Data"]
related_syntax: []
see_also: ["The Iterator trait", "Iterator adaptors", "Iterator consumers", "Custom iterators"]
---

## Explanation

Building an [adaptor](iterator-adaptors.md) chain does no work by itself.
Writing `data.iter().map(f).filter(g)` only constructs a small nested
struct describing what *would* happen to an item if one arrived — no
call to `f` or `g` actually happens at that line. Work only starts when
something pulls values through the chain: a `for` loop, or a
[consumer](iterator-consumers.md) like `sum` or `collect`. This is what
"lazy" means here — computation is deferred until it's demanded, rather
than run eagerly as each step is written.

This deferral is what makes infinite iterators usable at all. A range
like `0..` never terminates on its own, but because nothing runs until
requested, it's perfectly safe to build adaptors on top of it as long as
something downstream — `take`, `find`, or a manual `break` — decides when
to stop asking for more. An eager language would have to materialize the
whole sequence (or a huge prefix of it) before any transformation could
even be applied.

Laziness is also why long chains stay cheap: because no intermediate
collection is ever built between adaptors, the compiler can typically
fuse the whole chain into a single loop that computes one final item at a
time, directly from the source, with no allocation in between. This is
the flip side of the same property described in
[Iterator adaptors](iterator-adaptors.md) — the wrapping approach and the
laziness are two views of the same mechanism.

The most common gotcha follows directly from this: an adaptor chain built
for its side effects (say, a `map` closure that prints something) and
then never consumed will do nothing at all, silently. The chain isn't
broken — it was simply never asked to run. The upside of the same
property is architectural: a function can return `impl Iterator<Item =
T>` instead of an eagerly built `Vec<T>`, letting the caller decide how
much of the sequence to actually pay for.

## Basic usage example

```
let numbers = 1..; // an infinite range

let doubled = numbers.map(|n| n * 2); // <- map builds a pipeline here; nothing has been computed yet

let first_three: Vec<i32> = doubled.take(3).collect(); // work only happens here, when collect pulls values
assert_eq!(first_three, vec![2, 4, 6]);
```

## Best practices & deeper information

### Scenario: Working with collections

Searching a huge stream of generated reading ids for the first one past
a threshold only needs to compute as far as the first match — laziness
means the rest is never touched.

```
let reading_ids = (1..=1_000_000).map(|n| n * 10); // <- a lazy pipeline over a million ids; nothing has been computed yet

let first_over_threshold = reading_ids
    .filter(|&id| id > 500_000)
    .next(); // <- only pulls values until the first match, stopping early

assert_eq!(first_over_threshold, Some(500_010));
```

**Why this way:** because `map` and `filter` are lazy, `.next()` stops
the moment it finds a match instead of materializing a million-element
`Vec` first — the
[`Iterator` trait docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
describe adaptors as producing values "on demand" for exactly this
reason.

### Scenario: Designing a public API

A method that reports which sensor readings crossed a threshold doesn't
need to build a `Vec` of all of them if the caller only wants the first —
returning a lazy iterator lets the caller decide.

```
struct SensorLog {
    readings: Vec<f64>,
}

impl SensorLog {
    pub fn above_threshold(&self, threshold: f64) -> impl Iterator<Item = f64> + '_ {
        // <- returns a lazy iterator; the caller decides how much to actually consume
        self.readings.iter().copied().filter(move |&r| r > threshold)
    }
}

let log = SensorLog { readings: vec![68.1, 72.4, 75.0, 69.9] };
let spike = log.above_threshold(70.0).next();
assert_eq!(spike, Some(72.4));
```

**Why this way:** returning `impl Iterator<Item = f64>` instead of
`Vec<f64>` defers the cost of computing (and allocating) every match
until — and unless — the caller actually asks for it, an API design
principle [Effective Rust](https://effective-rust.com/) frames as
preferring the laziest return type that still satisfies every caller.

## Explanation (Embedded)

Laziness is a property of `core::iter`'s adaptors, not of `std`, so it
holds exactly as written on a `#![no_std]` target: building a chain with
`map`/`filter` does no work and allocates nothing until something pulls
values through it. This matters even more on a memory-constrained target
than on a hosted one — because no intermediate buffer is ever
materialized between adaptor stages, a long chain over a fixed-size array
of readings still costs nothing beyond the final values actually
produced, and the compiler typically fuses the whole chain into one loop.
Laziness also means a chain can be built over a source that's slow to
fully drain — a peripheral FIFO being drained one register read at a
time — and still be handed to a caller that only wants the first few
values, without ever pulling more reads than requested.

## Basic usage example (Embedded)

```
let register_bank: [u16; 6] = [10, 45, 90, 200, 512, 1023];

let scaled = register_bank.iter().map(|&r| r / 2); // <- map builds a pipeline; nothing computed yet, no allocation

let first_two: [u16; 2] = {
    let mut it = scaled.take(2); // still lazy
    [it.next().unwrap(), it.next().unwrap()] // work happens here, one value at a time
};
assert_eq!(first_two, [5, 22]);
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

Searching a buffer of recent sensor samples for the first one past a
threshold only needs to look as far as the first match — laziness means
the rest of the buffer is never touched, and nothing is ever collected
into a `Vec` to search through.

```
let samples: [u16; 8] = [480, 495, 510, 605, 490, 500, 512, 498];

let first_over_threshold = samples
    .iter()
    .filter(|&&s| s > 600) // <- filter is lazy: nothing evaluated until pulled
    .next(); // <- stops at the first match, ignoring the rest of the buffer

assert_eq!(first_over_threshold, Some(&605));
```

**Why this way:** stopping at the first match without ever building an
intermediate collection keeps the whole search on the stack with a
bounded, small number of comparisons — exactly the property that matters
when the buffer represents scarce peripheral reads rather than free
in-memory data.

### Scenario: Designing a public API

A driver function that reports which channel readings crossed a
threshold shouldn't build a `heapless::Vec` of every match if the caller
only wants the first — returning a lazy iterator over a borrowed fixed
buffer lets the caller decide how much to consume, with no allocation
forced either way.

```
struct ChannelLog {
    readings: [i16; 4],
}

impl ChannelLog {
    fn above_threshold(&self, threshold: i16) -> impl Iterator<Item = i16> + '_ {
        // <- returns a lazy iterator borrowing `readings`; caller decides how much to consume
        self.readings.iter().copied().filter(move |&r| r > threshold)
    }
}

let log = ChannelLog { readings: [18, 24, 30, 21] };
let first_spike = log.above_threshold(25).next();
assert_eq!(first_spike, Some(30));
```

**Why this way:** returning `impl Iterator<Item = i16>` instead of a
collected buffer defers work until the caller actually asks for it, and
because the iterator borrows the fixed array rather than owning a new
allocation, the whole API stays viable with zero heap on a `#![no_std]`
target.

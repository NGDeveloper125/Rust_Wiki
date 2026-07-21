---
title: "The Iterator trait"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Functional Programming", "Iterating & Transforming Data"]
related_syntax: [for, in]
see_also: ["IntoIterator (iter/iter_mut/into_iter)", "Iterator adaptors", "Iterator consumers", "Lazy evaluation", "Custom iterators", "Generics", "Closures & capturing"]
---

## Explanation

`Iterator` is the trait behind every sequential walk over data in Rust —
a `Vec`'s elements, a `HashMap`'s entries, a range of numbers, the lines
of a file, the bytes of a socket. Underneath all of that variety sits one
tiny contract: a type implementing `Iterator` declares an associated
`Item` type and provides a single required method, `next(&mut self) ->
Option<Self::Item>`, which produces one item at a time and returns `None`
once there's nothing left. Everything else — `map`, `filter`, `sum`,
`collect`, and dozens more — is a default method built entirely on top of
that one method, provided free once `next` exists.

This is why the trait matters as an abstraction, not just a convenience:
code written against `Iterator` doesn't need to know or care whether the
values are coming from a `Vec`, a `BTreeMap`, a parsed text stream, or a
value being computed on the fly. A `for` loop, a generic function bounded
by `Iterator<Item = T>`, or an adaptor chain all work identically
regardless of the source, because the source is hidden behind this one
interface. Contrast this with index-based loops (`for i in
0..collection.len()`), which tie the loop to a specific way of accessing
elements and don't generalize to sources that have no meaningful index at
all, like a channel receiver or a line-by-line file reader.

The mental model worth keeping is: an iterator is a stateful cursor, not
the data itself. Calling `next()` advances that cursor and hands back
whatever it currently points at; the underlying collection (if there is
one) is untouched by this process unless the iterator was created in a
way that consumes it. Getting hold of an iterator in the first place —
deciding whether to borrow, mutably borrow, or take ownership of the
source — is a separate concern, covered by
[IntoIterator](intoiterator.md).

The trait's transformation and consumption methods split into two
families worth knowing by name: [adaptors](iterator-adaptors.md), which
wrap one iterator in another and stay [lazy](lazy-evaluation.md) until
something asks for a value, and [consumers](iterator-consumers.md),
which actually drive the cursor to produce a final result. Any type can
join this whole ecosystem for free by implementing just `next` — see
[Custom iterators](custom-iterators.md) — which is also how the standard
library itself builds `map`, `filter`, and the rest, each one just
another small struct implementing `Iterator` around an inner iterator.

## Basic usage example

```
let mut numbers = [10, 20, 30].iter();

assert_eq!(numbers.next(), Some(&10)); // <- next() is the trait's one required method
assert_eq!(numbers.next(), Some(&20));
```

## Best practices & deeper information

### Scenario: Working with collections

Scanning a batch of sensor readings for spikes only needs the one method
every `Iterator` provides — the loop doesn't touch the `Vec` itself, only
the cursor walking across it.

```
let readings = vec![72.4, 68.1, 75.0, 69.9];
let mut iter = readings.iter(); // <- `iter` is a value implementing Iterator; the Vec itself is untouched

let mut spikes = 0;
while let Some(&reading) = iter.next() { // <- next() drives iteration one item at a time
    if reading > 70.0 {
        spikes += 1;
    }
}
println!("{spikes} readings above threshold");
```

**Why this way:** every collection's `.iter()` hands back a type
implementing `Iterator`, so code that only needs `next()` works
identically whether the source was a `Vec`, a slice, or something else
entirely — the
[`Iterator` trait docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
describe `next` as the trait's sole required method for exactly this
reason.

### Scenario: Working with text

Splitting a log line into fields produces something implementing
`Iterator` just like a collection would, even though no collection was
ever built.

```
let log_line = "2026-07-20 ERROR disk_usage_high";
let mut fields = log_line.split_whitespace(); // <- split_whitespace returns something implementing Iterator

let date = fields.next(); // <- next() pulls one field at a time, regardless of how splitting works internally
let level = fields.next();
assert_eq!(date, Some("2026-07-20"));
assert_eq!(level, Some("ERROR"));
```

**Why this way:** `str` methods like `split_whitespace`, `chars`, and
`lines` all return distinct concrete types, but every one of them
implements `Iterator`, so callers rely on one interface rather than
learning a bespoke API per method — the
[standard library docs](https://doc.rust-lang.org/std/primitive.str.html#method.split_whitespace)
document each of these as returning "an iterator."

### Scenario: Writing generic code

A helper that finds the highest sensor reading shouldn't care whether the
readings come from a `Vec`, an array, or a filtered chain — it only needs
something that implements `Iterator<Item = f64>`.

```
fn max_reading<I>(readings: I) -> Option<f64>
where
    I: Iterator<Item = f64>, // <- bound directly on the Iterator trait, not on any concrete collection
{
    readings.fold(None, |max, r| match max {
        Some(m) if m >= r => Some(m),
        _ => Some(r),
    })
}

let highest = max_reading(vec![72.4, 68.1, 75.0].into_iter());
assert_eq!(highest, Some(75.0));
```

**Why this way:** bounding the parameter on `Iterator<Item = f64>` rather
than `Vec<f64>` lets the function accept any source that can produce
`f64`s — an array, a filtered chain, a channel receiver — without
changing its signature, the flexibility the
[Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
recommends generic bounds for.

## Explanation (Embedded)

`Iterator` is defined in `core::iter`, not `std::iter` — the trait, its
associated `Item` type, and the required `next` method have no dependency
on an allocator, a heap, or an operating system. On a `#![no_std]`
embedded target, iterating a fixed-size array of sensor readings, a
memory-mapped register range, or a `heapless::Vec` uses exactly the same
`next(&mut self) -> Option<Self::Item>` contract as hosted Rust — nothing
about the trait itself changes. This matters more in embedded than it
first sounds: because the whole adaptor/consumer machinery built on top
of `next` is zero-cost, an iterator chain over a fixed array compiles
down to the same tight loop a hand-written index-based loop would, with
no runtime overhead and no hidden allocation — a genuine embedded selling
point, not just a portability footnote. The one thing to keep separate
from the trait itself is what a *consumer* does with the items it pulls
out; `sum`, `count`, and `fold` need nothing beyond `core`, while
`collect` into an allocating type is a separate concern (see
[FromIterator & collect targets](fromiterator-and-collect-targets.md)).

## Basic usage example (Embedded)

```
let readings: [u16; 4] = [512, 498, 610, 523]; // raw ADC samples

let mut samples = readings.iter();
assert_eq!(samples.next(), Some(&512)); // <- next() is the same required method as hosted Rust
assert_eq!(samples.next(), Some(&498));
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

Scanning a fixed-size buffer of ADC samples for a value past a threshold
only needs `next()` — no heap, no `Vec`, since the buffer is a plain
array living on the stack (or in `.bss`).

```
let samples: [u16; 8] = [512, 498, 610, 523, 700, 480, 505, 690];
let mut iter = samples.iter(); // <- iterating a plain array; no allocation involved

let mut over_threshold = 0;
while let Some(&sample) = iter.next() { // <- next() drives iteration one sample at a time
    if sample > 600 {
        over_threshold += 1;
    }
}
assert_eq!(over_threshold, 2);
```

**Why this way:** a fixed `[u16; N]` array's `.iter()` returns a type
implementing `Iterator` exactly like a `Vec`'s would, so the same
one-required-method contract applies whether the readings live on the
stack of a microcontroller or in a heap-backed collection on a hosted
target.

### Scenario: Writing generic code

A helper that finds the highest reading from a bank of registers
shouldn't care whether the values came from a fixed array, a
`heapless::Vec`, or a peripheral read — bounding it on `Iterator` keeps it
reusable across all three, with no allocation pulled in by the bound
itself.

```
fn max_reading<I>(readings: I) -> Option<u16>
where
    I: Iterator<Item = u16>, // <- bound directly on core::iter's Iterator, no alloc required
{
    readings.fold(None, |max, r| match max {
        Some(m) if m >= r => Some(m),
        _ => Some(r),
    })
}

let register_bank: [u16; 3] = [512, 498, 610];
let highest = max_reading(register_bank.into_iter());
assert_eq!(highest, Some(610));
```

**Why this way:** bounding on `Iterator<Item = u16>` instead of `&[u16]`
or a concrete `heapless::Vec<u16, N>` lets the same function accept a
fixed array, a `heapless` collection, or a filtered chain over either,
without pulling `alloc` into a function that never needed it.

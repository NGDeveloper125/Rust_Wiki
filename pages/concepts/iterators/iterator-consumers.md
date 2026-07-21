---
title: "Iterator consumers"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Functional Programming", "Iterating & Transforming Data"]
related_syntax: []
see_also: ["The Iterator trait", "Iterator adaptors", "FromIterator & collect targets", "Lazy evaluation"]
---

## Explanation

A consumer is a method on [`Iterator`](the-iterator-trait.md) that drives
iteration to completion and produces a final, non-iterator value —
`collect`, `sum`, `fold`, `reduce`, `count`, `for_each`, `any`/`all`, and
`min`/`max` are all consumers. Where an [adaptor](iterator-adaptors.md)
wraps an iterator in another iterator and does nothing by itself, a
consumer is the thing that actually calls `next()` repeatedly and does
something with what comes back — it's the point where a
[lazily built](lazy-evaluation.md) chain finally runs.

Consumers split into two shapes worth telling apart: some exhaust the
entire iterator no matter what — `sum`, `count`, and `collect` have to see
every item to give a correct answer — while others short-circuit,
stopping as soon as the answer is known. `any` returns `true` (and stops)
at the first item satisfying its predicate; `all` returns `false` (and
stops) at the first item that doesn't. Choosing the consumer that
matches the actual question ("does one exist?" versus "how many exist?")
avoids walking further than necessary.

`fold` is the general-purpose consumer underneath many of the others: it
carries an accumulator through the whole iterator, updating it once per
item, and returns the final accumulator. `sum`, `count`, and `min`/`max`
are effectively named special cases of the same idea for common
accumulations. `reduce` is `fold` without a separate initial value — it
uses the iterator's own first item as the starting accumulator, which
only makes sense when the accumulator and the item are the same type.

`collect` is technically a consumer too — it drives the iterator to
completion just like the others — but what it builds depends on a whole
trait of its own; see
[FromIterator & collect targets](fromiterator-and-collect-targets.md) for
that side of it. And because a consumer only knows how to call `next()`,
implementing [a custom iterator](custom-iterators.md) is all it takes for
every consumer here to become usable on your own type for free.

## Basic usage example

```
let shipment_weights_kg = [12.5, 8.0, 20.25];

let total: f64 = shipment_weights_kg.iter().sum(); // <- sum is a consumer: drives iteration to completion
assert_eq!(total, 40.75);
```

## Best practices & deeper information

### Scenario: Working with collections

Checking user records for how many are active, and whether every
username is non-empty, calls for two different consumers matched to the
two different questions being asked.

```
struct User { username: String, active: bool }

let users = vec![
    User { username: "asha".into(), active: true },
    User { username: "beto".into(), active: false },
    User { username: "cleo".into(), active: true },
];

let active_count = users.iter().filter(|u| u.active).count(); // <- count: a consumer, drives the chain to a final number
let all_named = users.iter().all(|u| !u.username.is_empty()); // <- all: a consumer, short-circuits on the first failure

assert_eq!(active_count, 2);
assert!(all_named);
```

**Why this way:** `count` has to see every item, but `all` stops the
instant it finds a violation — picking the consumer that matches "how
many?" versus "does every one satisfy this?" is more direct than
collecting into a `Vec` first, per the
[`Iterator` trait docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.all).

### Scenario: Numeric computation

Averaging shipment weights needs both a sum and a count, computed
together in one pass with `fold` rather than walking the data twice.

```
let shipment_weights_kg = [12.5, 8.0, 20.25, 5.0];

let (total, count) = shipment_weights_kg
    .iter()
    .fold((0.0, 0), |(sum, n), &w| (sum + w, n + 1)); // <- fold: a consumer, accumulates a running (sum, count) pair

let average = total / count as f64;
assert!((average - 11.4375).abs() < 0.001);
```

**Why this way:** `fold` accumulates both figures in a single pass over
the data instead of calling `.sum()` and `.count()` separately (which
would each walk the whole slice on their own), the general-purpose
accumulation pattern the
[`Iterator::fold` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)
describe.

### Scenario: Working with text

Finding the longest of several log lines is a natural fit for `reduce`,
since the running "longest so far" is the same type as each line being
compared.

```
let log_lines = ["INFO start", "ERROR disk_full", "WARN retrying"];

let longest = log_lines
    .iter()
    .copied()
    .reduce(|a, b| if b.len() > a.len() { b } else { a }); // <- reduce: a consumer, folds without a separate initial value

assert_eq!(longest, Some("ERROR disk_full"));
```

**Why this way:** `reduce` avoids inventing an artificial initial value
(there's no natural "empty" line to seed a `fold` with) by using the
iterator's first item as the seed instead, exactly the case the
[`Iterator::reduce` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.reduce)
recommend it for over `fold`.

## Explanation (Embedded)

Consumers — `sum`, `count`, `fold`, `reduce`, `for_each`, `any`/`all`,
`min`/`max` — are default methods defined in `core::iter`, so they run
identically on a `#![no_std]` target: none of them need an allocator,
because each only needs to hold a running accumulator (a number, a
`bool`, a running max) rather than build a new collection. This makes
them a natural fit for reading a fixed-size buffer of sensor samples or
scanning a bank of registers — a firmware loop that used to hand-roll an
accumulator variable and a `for` loop over indices can instead reach for
`.sum()`, `.fold()`, or `.any()` at no cost difference. `collect` is the
one exception worth flagging on its own: it's still a consumer, but
*what* it collects into determines whether it needs `alloc` at all — see
[FromIterator & collect targets](fromiterator-and-collect-targets.md) for
that caveat.

## Basic usage example (Embedded)

```
let shipment_weights_g: [u32; 3] = [1250, 800, 2025]; // grams, fixed-capacity buffer

let total: u32 = shipment_weights_g.iter().sum(); // <- sum is a consumer: no allocation, just an accumulator
assert_eq!(total, 4075);
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

Counting how many entries in a fixed-size buffer of register readings
are non-zero, and checking whether every one is within range, calls for
two consumers matched to the two questions — with no `Vec` built to hold
intermediate results.

```
let register_values: [u16; 4] = [0, 512, 498, 610];

let nonzero_count = register_values.iter().filter(|&&v| v != 0).count(); // <- count: consumer, drives to a final number
let all_in_range = register_values.iter().all(|&v| v <= 1023); // <- all: consumer, short-circuits on first violation

assert_eq!(nonzero_count, 3);
assert!(all_in_range);
```

**Why this way:** `count` and `all` each walk the fixed array directly
and produce a plain number or `bool`, so neither needs a heap-backed
intermediate — a meaningful distinction on a target with kilobytes of
RAM rather than gigabytes.

### Scenario: Numeric computation

Computing a running average of buffered sensor samples needs both a sum
and a count from a single pass — `fold` does this with one accumulator
tuple and no second walk over the buffer.

```
let samples: [i32; 4] = [21, 23, 19, 25]; // degrees C, fixed buffer from a ring of recent readings

let (total, count) = samples
    .iter()
    .fold((0, 0), |(sum, n), &s| (sum + s, n + 1)); // <- fold: consumer, accumulates (sum, count) with no allocation

let average = total / count;
assert_eq!(average, 22);
```

**Why this way:** `fold` walks the fixed buffer exactly once, keeping its
whole accumulator in a couple of registers or stack slots — important
when the alternative (`.sum()` then `.count()` separately) would mean a
second pass over memory that may be scarce.

---
title: "Iterator adaptors"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Functional Programming", "Iterating & Transforming Data", "Coming from Python / JavaScript", "Coming from Haskell / functional languages"]
related_syntax: []
see_also: ["The Iterator trait", "Iterator consumers", "Lazy evaluation", "Closures & capturing", "Higher-order functions"]
---

## Explanation

An adaptor is a method on [`Iterator`](the-iterator-trait.md) that takes
an iterator and returns a new iterator — `map`, `filter`, `zip`,
`enumerate`, `chain`, `flat_map`, `rev`, `scan`, `peekable`, `take`, and
`skip` are all adaptors. Each one wraps the iterator it's called on in a
small struct that knows how to compute its own `next()` by pulling from
and transforming the iterator underneath it. Crucially, none of them
produce a final value or run any of that computation themselves — they
only describe a transformation, which is why they're covered in depth
alongside [lazy evaluation](lazy-evaluation.md).

The mental model is an assembly line: `data.iter().map(f).filter(g)`
builds a chain of small wrapper stations, each one describing what it
will do to an item once an item actually arrives, without touching any
data yet. Nothing runs until something at the end pulls a value through
the whole chain — see [Iterator consumers](iterator-consumers.md) for
what that "something" usually is.

This wrapping approach is what makes long adaptor chains both readable
and cheap: because every adaptor both consumes an `Iterator` and produces
one, chains of arbitrary length still type-check as a single nested type,
and the compiler typically inlines the whole chain into a tight loop with
no intermediate collection allocated at each step — very different from
writing `.map(f).collect::<Vec<_>>()` and then `.filter(g)` on the
result, which forces an allocation in between for no reason.

Adaptors like `map` and `filter` take a closure describing the per-item
transformation or condition — see
[Closures & capturing](../functions-closures/closures-and-capturing.md) for how those closures
capture their environment. Because an adaptor chain by itself does
nothing observable, forgetting to attach a consumer (or a `for` loop) at
the end of one is a common source of "why didn't this run?" bugs; the
compiler will usually warn about an unused value in that case, but it's
worth knowing the chain was never broken — it was simply never asked to
produce anything.

## Basic usage example

```
let prices = [1500, 4200, 800]; // cents

let discounted: Vec<i32> = prices.iter().map(|p| p - 100).collect(); // <- map is an adaptor: transforms each item lazily
```

## Best practices & deeper information

### Scenario: Working with collections

Reporting only the money from orders that weren't cancelled reads as a
`filter` narrowing the stream followed by a `map` reshaping what's left,
instead of a loop with an `if` and a running `Vec`.

```
struct Order { id: u32, total_cents: u32, cancelled: bool }

let orders = vec![
    Order { id: 1, total_cents: 4200, cancelled: false },
    Order { id: 2, total_cents: 1500, cancelled: true },
    Order { id: 3, total_cents: 9900, cancelled: false },
];

let active_totals: Vec<u32> = orders
    .iter()
    .filter(|o| !o.cancelled) // <- filter: an adaptor, skips cancelled orders lazily
    .map(|o| o.total_cents)   // <- map: an adaptor, transforms Order into just its total
    .collect();

assert_eq!(active_totals, vec![4200, 9900]);
```

**Why this way:** chaining `filter` then `map` keeps each step doing one
job and reads in the same order the data flows, an idiom the
[Rust Cookbook's iterator recipes](https://rust-lang-nursery.github.io/rust-cookbook/algorithms/sorting.html)
build on throughout.

### Scenario: Working with text

Tagging each word of a log line with its position combines splitting text
with `enumerate`, without a manually incremented counter.

```
let log_line = "warn disk_usage_high retry_count_3";

let tagged: Vec<(usize, &str)> = log_line
    .split_whitespace()
    .enumerate() // <- enumerate: an adaptor, pairs each word with its position
    .collect();

assert_eq!(tagged[1], (1, "disk_usage_high"));
```

**Why this way:** `enumerate` removes the need for a hand-rolled index
variable, which the
[`Iterator::enumerate` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.enumerate)
note is both less error-prone and reads closer to the intent ("each item,
with its position") than tracking an index alongside the loop.

### Scenario: Numeric computation

Turning a series of hourly readings into a running total needs an
adaptor that carries state between items — plain `map` can't do that on
its own.

```
let hourly_readings = [2.0, 3.5, -1.0, 4.0];

let running_totals: Vec<f64> = hourly_readings
    .iter()
    .scan(0.0, |total, &r| { // <- scan: an adaptor carrying state across each step
        *total += r;
        Some(*total)
    })
    .collect();

assert_eq!(running_totals, vec![2.0, 5.5, 4.5, 8.5]);
```

**Why this way:** `scan` is the adaptor built specifically for
accumulating state while still yielding one output per input item, per
the
[`Iterator::scan` docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.scan) —
using `fold` here would only produce the final total, not each running
step.

## Explanation (Embedded)

Adaptors (`map`, `filter`, `zip`, `enumerate`, `scan`, `take`, and the
rest) are defined in `core::iter`, so building a chain of them costs
nothing beyond the code itself — no allocator, no heap. This is one of
the more concrete zero-cost-abstraction wins embedded Rust gets from the
language: `sensor_readings.iter().filter(|&&r| r > threshold).map(|&r| r
* scale)` compiles to the same tight loop a hand-rolled `for` loop with
an `if` inside it would, with no intermediate array or `Vec` materialized
between the `filter` and the `map`. The chain is just a nested struct
type; the compiler inlines and fuses it. The one part of a chain that can
pull in `alloc` is a *consumer* at the end (like `.collect::<Vec<_>>()`)
— the adaptors in between never do, so a chain over a fixed-size array or
a `heapless::Vec` can be built as long or as elaborate as needed while
staying entirely on the stack.

## Basic usage example (Embedded)

```
let raw_samples: [u16; 4] = [512, 498, 610, 523]; // raw ADC counts

let scaled_over_threshold: u32 = raw_samples
    .iter()
    .filter(|&&s| s > 500) // <- filter: an adaptor, no allocation
    .map(|&s| s as u32 * 3) // <- map: an adaptor, no allocation
    .sum(); // consumer: drives the chain, still no heap involved

assert_eq!(scaled_over_threshold, (512 + 610 + 523) * 3);
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

Reporting the average of only the in-range readings from a fixed sensor
buffer reads as `filter` then a numeric accumulation, with no `Vec` built
at any point.

```
let readings: [i16; 6] = [-40, 22, 25, 130, 24, 23]; // last is a spurious out-of-range spike

let in_range_avg: i32 = {
    let mut sum = 0i32;
    let mut count = 0i32;
    for r in readings.iter().filter(|&&r| (-20..100).contains(&r)) { // <- filter: adaptor, skips out-of-range spikes
        sum += *r as i32;
        count += 1;
    }
    sum / count
};

assert_eq!(in_range_avg, 23);
```

**Why this way:** `filter` discards the spurious `130` reading before the
loop ever sees it, and because adaptors allocate nothing, this pattern is
as safe to use on a memory-constrained microcontroller as a manual bounds
check would be — with none of the manual bookkeeping.

### Scenario: Bit manipulation and flags

Extracting which channels of a status register have their "ready" bit
set combines a fixed array of raw register words with `enumerate` and
`filter`, entirely without allocation.

```
let status_words: [u8; 4] = [0b0000_0001, 0b0000_0000, 0b0000_0001, 0b0000_0001];

let ready_channels: [bool; 4] = {
    let mut flags = [false; 4];
    for (i, word) in status_words.iter().enumerate() { // <- enumerate: adaptor, pairs each register word with its index
        flags[i] = word & 0b1 != 0; // READY bit is bit 0
    }
    flags
};

assert_eq!(ready_channels, [true, false, true, true]);
```

**Why this way:** `enumerate` gives each register word its channel index
without a hand-tracked counter, and writing results into a fixed `[bool;
4]` instead of collecting into a `Vec` keeps the whole scan on the stack
— the same adaptor used on a hosted `Vec<u8>` in the classic example
works unchanged here.

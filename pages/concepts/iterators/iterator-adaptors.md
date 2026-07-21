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

## Embedded Rust Notes

**Full support for building the chain.** Adaptors themselves live in
`core::iter` and allocate nothing — `map`, `filter`, `scan`, and the rest
work identically on a `#![no_std]` target. Only the final consumer at the
end of a chain might need `alloc` (for example, `collect()` into a `Vec`
or `String`); the adaptors in between never do.

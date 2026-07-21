---
title: "Custom iterators"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Iterating & Transforming Data"]
related_syntax: []
see_also: ["The Iterator trait", "IntoIterator (iter/iter_mut/into_iter)", "Iterator adaptors", "Traits"]
---

## Explanation

Any type can join the entire [`Iterator`](the-iterator-trait.md)
ecosystem by implementing the trait for itself: declare an associated
`Item` type and provide `next(&mut self) -> Option<Self::Item>`. That's
the whole requirement — every [adaptor](iterator-adaptors.md) (`map`,
`filter`, `zip`, …) and every [consumer](iterator-consumers.md) (`sum`,
`collect`, `fold`, …) is a default method the trait provides on top of
`next`, so a type gains all of them the moment `next` exists, without
writing another line.

Implementing `Iterator` directly on a type is different from
implementing [`IntoIterator`](intoiterator.md) on a *container* type that
holds items internally. A container usually shouldn't implement
`Iterator` itself — instead, it implements `IntoIterator`, and hands back
a small, separate iterator struct (often just holding a cursor or index
into the container) that implements `Iterator`. This is exactly the
pattern the standard library follows: `Vec<T>` doesn't implement
`Iterator`; calling `.iter()` on it produces a distinct `Iter` type that
does.

A custom iterator's own fields are its entire state — whatever `next()`
needs to know to produce the following item and recognize when it's
done. A countdown keeps a remaining count; a line reader keeps a
position; a graph traversal keeps a stack or queue of nodes still to
visit. This is the same "where am I" bookkeeping a manual indexed loop
would otherwise track by hand, just packaged behind `next()` instead of
scattered through a loop body.

The standard library's own adaptors are built the same way: `Map`,
`Filter`, and the rest are each an ordinary struct wrapping an inner
iterator, implementing `Iterator` by pulling from that inner iterator and
transforming what comes back. Writing a custom iterator that wraps
another one (rather than a fresh data source) is the same technique,
just applied to your own logic instead of the standard library's.

## Basic usage example

```
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> { // <- the one method a custom iterator must implement
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

let launch: Vec<u32> = Countdown(3).collect();
assert_eq!(launch, vec![3, 2, 1]);
```

## Best practices & deeper information

### Scenario: Implementing traits

Generating retry delays that double each time is a natural fit for a
hand-written `Iterator` implementation rather than a precomputed list —
each delay only needs to exist once it's actually asked for.

```
struct RetryBackoff {
    delay_ms: u64,
    attempts_left: u32,
}

impl Iterator for RetryBackoff { // <- Iterator implemented for a custom type, not derived
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.attempts_left == 0 {
            return None;
        }
        self.attempts_left -= 1;
        let current = self.delay_ms;
        self.delay_ms *= 2;
        Some(current)
    }
}

let delays: Vec<u64> = RetryBackoff { delay_ms: 100, attempts_left: 4 }.collect();
assert_eq!(delays, vec![100, 200, 400, 800]);
```

**Why this way:** implementing `Iterator` directly makes `RetryBackoff`
work with every existing adaptor and consumer for free, and the type is
named after exactly what it produces, following the
[API Guidelines' iterator-naming convention (C-ITER-TY)](https://rust-lang.github.io/api-guidelines/naming.html#iterator-type-names-match-the-function-that-produces-them-c-iter-ty).

### Scenario: Working with collections

A small ring buffer type shouldn't implement `Iterator` on itself —
instead it hands out a separate iterator struct that holds a cursor into
its data, exactly the way `Vec::iter()` does.

```
struct RingBuffer {
    items: Vec<f64>,
}

struct RingBufferIter<'a> {
    items: &'a [f64],
    pos: usize,
}

impl<'a> Iterator for RingBufferIter<'a> { // <- a second, small iterator type wrapping a slice cursor
    type Item = &'a f64;

    fn next(&mut self) -> Option<&'a f64> {
        let item = self.items.get(self.pos)?;
        self.pos += 1;
        Some(item)
    }
}

impl RingBuffer {
    fn iter(&self) -> RingBufferIter<'_> {
        RingBufferIter { items: &self.items, pos: 0 }
    }
}

let buffer = RingBuffer { items: vec![19.5, 20.1, 18.7] };
let average: f64 = buffer.iter().sum::<f64>() / buffer.items.len() as f64;
println!("{average:.2}");
```

**Why this way:** pairing a container with its own small cursor-holding
iterator type is exactly how `Vec` and `HashMap` expose `.iter()` in the
standard library, a shape the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) book
points to as the idiomatic way to make a custom collection iterable
without forcing callers to clone into a `Vec` first.

### Scenario: Designing a public API

A library function that produces retry delays shouldn't force callers to
name the concrete iterator type — returning `impl Iterator` lets the
internal implementation change later without breaking anyone.

```
mod backoff {
    pub struct RetryBackoff {
        delay_ms: u64,
        attempts_left: u32,
    }

    impl Iterator for RetryBackoff {
        type Item = u64;

        fn next(&mut self) -> Option<u64> {
            if self.attempts_left == 0 {
                return None;
            }
            self.attempts_left -= 1;
            let current = self.delay_ms;
            self.delay_ms *= 2;
            Some(current)
        }
    }

    pub fn retry_delays(attempts: u32) -> impl Iterator<Item = u64> {
        // <- the concrete RetryBackoff type stays private; callers only see "some Iterator"
        RetryBackoff { delay_ms: 50, attempts_left: attempts }
    }
}

let delays: Vec<u64> = backoff::retry_delays(3).collect();
assert_eq!(delays, vec![50, 100, 200]);
```

**Why this way:** hiding the concrete `RetryBackoff` type behind `impl
Iterator<Item = u64>` means the crate is free to change how the delays
are generated later without it counting as a breaking change, the kind of
extensibility the
[API Guidelines' future-proofing section](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends designing for up front.

## Embedded Rust Notes

**Full support.** Implementing `Iterator` for a plain struct is core
language and requires no allocator — a countdown, a fixed-size buffer
cursor, or a hardware register scanner can all implement it directly on
a `#![no_std]` target. The caveat is in the *contents* of the iterator,
not the trait itself: an iterator struct that owns a heap-allocated `Vec`
(as in the ring buffer example) needs `alloc` or a `heapless`-style
fixed-capacity equivalent.

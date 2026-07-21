---
title: "<"
kind: operator
embedded_support: full
groups: [Comparison, Basics, "Types & Data Structures"]
related_concepts: [Operator overloading, Generics]
related_syntax: [">", "<=", ">="]
see_also: [">"]
---

## Explanation

`<` is the less-than comparison, overloadable via `std::cmp::PartialOrd`.

`<` is also the opening delimiter for **generic parameter lists**
(`Vec<T>`, `fn f<T>()`) — an entirely different, non-operator role. This
dual use is why the parser sometimes needs help disambiguating generics
from a chained comparison (`a < b, c > d` reads ambiguously); the
"turbofish" `::<...>` exists specifically to disambiguate generics in
expression position (see [`::`](path-separator.md)).

## Usage examples

### Checking whether one value is less than another

```
let a = 3;
let b = 5;
let smaller = a < b; // <- true if `a` is less than `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a < b < c` doesn't compile; write `a < b && b < c` instead.

### Working with collections

When a sort needs custom logic — here, ordering support tickets by
priority — `sort_by` takes a comparator where `<` defines what "comes
first" means for that call.

```
struct Ticket {
    id: u32,
    priority: u8,
}

let mut tickets = vec![
    Ticket { id: 101, priority: 3 },
    Ticket { id: 102, priority: 1 },
    Ticket { id: 103, priority: 2 },
];

tickets.sort_by(|a, b| {
    if a.priority < b.priority { // <- `<` defines the Less case for this comparator
        std::cmp::Ordering::Less
    } else if a.priority > b.priority {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
});
```

[`Vec::sort_by`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_by)
gives full control over the ordering when a type doesn't derive `Ord` or
the sort key needs custom logic (the expanded if/else comparator here is
written out to spotlight `<` itself — `a.priority.cmp(&b.priority)` or
`sort_by_key` is the usual shorthand); prefer `sort()`/`sort_by_key()`
instead whenever a natural, derivable ordering already exists.

### Validating input

Bounds checks in Rust are conventionally exclusive at the top — a valid
index is anything strictly less than the collection's length.

```
struct Buffer {
    data: Vec<u8>,
}

fn is_valid_index(buffer: &Buffer, index: usize) -> bool {
    index < buffer.data.len() // <- `<` here is an exclusive upper bound: valid indices are 0..len
}
```

`index < len` (not `<=`) matches the zero-based,
exclusive-at-the-top convention that std's own range types and the
`Index` panic message use, per the
[std slice docs](https://doc.rust-lang.org/std/primitive.slice.html) —
using `<=` here would be an off-by-one bug.

## Explanation (Embedded)

`<` means the same thing under `#![no_std]` — `core::cmp::PartialOrd`,
and generics/turbofish stay pure compile-time grammar regardless of
target. The single most common `<` in firmware is a bounds check on a
fixed-size, stack-allocated buffer rather than a heap-growable `Vec` —
embedded code almost always indexes a `[T; N]` or a `heapless::Vec`
whose capacity is fixed at compile time, so `index < capacity` is the
everyday guard before every write.

## Usage examples (Embedded)

### Bounds-checking an index into a fixed-size sample buffer

```
const SAMPLE_BUFFER_LEN: usize = 32;

fn can_write_sample(write_index: usize) -> bool {
    write_index < SAMPLE_BUFFER_LEN // <- `<` confirms the index still lands inside the fixed-size buffer
}
```

### Flagging an undervoltage reading

```
const MIN_SAFE_MILLIVOLTS: u16 = 2700;

fn is_undervoltage(voltage_millivolts: u16) -> bool {
    voltage_millivolts < MIN_SAFE_MILLIVOLTS // <- `<` flags a reading that has dropped below the safe minimum
}
```

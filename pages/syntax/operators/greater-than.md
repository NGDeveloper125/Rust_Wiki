---
title: ">"
kind: operator
embedded_support: full
groups: [Comparison, Basics, "Types & Data Structures"]
related_concepts: [Operator overloading, Generics]
related_syntax: ["<", "<=", ">="]
see_also: ["<"]
---

## Explanation

`>` is the greater-than comparison, overloadable via `std::cmp::PartialOrd`.

Like `<`, `>` doubles as the **closing** delimiter for a generic parameter
list (`Vec<T>`) — for nested generics like `Vec<Vec<T>>`, the parser
splits the `>>` token into two closing angle brackets itself, so no space
is needed between them.

## Usage examples

### Checking whether one value is greater than another

```
let a = 5;
let b = 3;
let bigger = a > b; // <- true if `a` is greater than `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a > b > c` doesn't compile; write `a > b && b > c` instead.

### Working with collections

`f64` doesn't implement `Ord` (because of `NaN`), so finding the largest
of several floating-point totals means comparing manually with `>`
inside a fold instead of calling `Iterator::max()` directly.

```
struct Order {
    id: u32,
    total: f64,
}

let orders = vec![
    Order { id: 1, total: 42.50 },
    Order { id: 2, total: 108.25 },
    Order { id: 3, total: 76.00 },
];

let largest = orders.iter().reduce(|a, b| if b.total > a.total { b } else { a }); // <- `>` picks the running max
println!("{:?}", largest.map(|o| o.id));
```

A `reduce`/`fold` with a manual `>` comparison is one
way to find an extreme value over types without a total order; the
standard tools for floats are
`max_by(|a, b| a.total.partial_cmp(&b.total).unwrap())` or
`f64::total_cmp` as the comparator. See [`<`](less-than.md) for sorting
the same kind of data with a comparator instead of just finding one
extreme.

## Explanation (Embedded)

`>` means the same thing under `#![no_std]` — `PartialOrd` lives in
`core::cmp`, and its role as the closing delimiter for a generic
parameter list is pure compile-time grammar either way. What shows up
constantly in embedded code is a strict "has this crossed a limit"
check — a sensor reading that has gone past a safety threshold, or an
incoming frame that's larger than a fixed-size buffer can hold — checked
in a polling loop far more often than in hosted code, since there's
frequently no interrupt or event system to raise the condition instead.

## Usage examples (Embedded)

### Flagging an over-temperature reading

```
const MAX_SAFE_CELSIUS: i16 = 85;

fn is_overheating(temperature_celsius: i16) -> bool {
    temperature_celsius > MAX_SAFE_CELSIUS // <- `>` flags a reading that has crossed the safety limit
}

assert!(is_overheating(90));
assert!(!is_overheating(40));
```

### Rejecting a frame too large for a fixed-size receive buffer

```
const RX_BUFFER_LEN: usize = 128;

fn incoming_frame_too_large(frame_len: usize) -> bool {
    frame_len > RX_BUFFER_LEN // <- `>` rejects any frame that wouldn't fit the fixed-size receive buffer
}
```

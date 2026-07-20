---
title: ">"
kind: operator
embedded_support: full
groups: [Basics, "Types & Data Structures"]
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

## Basic usage example

```
let a = 5;
let b = 3;
let bigger = a > b; // <- true if `a` is greater than `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a > b > c` doesn't compile; write `a > b && b > c` instead.

## Best practices & deeper information

### Scenario: Working with collections

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

**Why this way:** a `reduce`/`fold` with a manual `>` comparison is one
way to find an extreme value over types without a total order; the
standard tools for floats are
`max_by(|a, b| a.total.partial_cmp(&b.total).unwrap())` or
`f64::total_cmp` as the comparator. See [`<`](less-than.md) for sorting
the same kind of data with a comparator instead of just finding one
extreme.

## Embedded Rust Notes

**Full support.** Same as [`<`](less-than.md) — `core::cmp`, no `std`
dependency.

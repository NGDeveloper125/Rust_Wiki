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

`>` is the greater-than comparison, overloadable via `std::ops::PartialOrd`.

```
if a > b { ... }
```

Like `<`, `>` doubles as the **closing** delimiter for a generic parameter
list (`Vec<T>`), which is the more common source of parser-ambiguity
complaints — nested generics like `Vec<Vec<T>>` used to require a space
before Rust's parser was taught to split `>>` into two closing angle
brackets itself (no space needed in modern Rust).

## Basic usage example

```
let a = 5;
let b = 3;
let bigger = a > b; // <- true if `a` is greater than `b`
```

**Restriction:** comparisons can't be chained like in Python —
`a > b > c` doesn't compile; write `a > b && b > c` instead.

## Embedded Rust Notes

**Full support.** Same as [`<`](less-than.md) — `core::cmp`, no `std`
dependency.

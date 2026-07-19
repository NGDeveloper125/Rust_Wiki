---
title: "else"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language, Pattern Matching]
related_syntax: [if, let]
see_also: [if]
---

## Explanation

`else` supplies the alternative branch to an `if` (or the diverging branch
of a `let ... else`). It cannot appear on its own — it is always the tail
of an `if` or `let` construct.

```
if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}
```

Chained `else if` is not special syntax — it's just an `if` expression
nested directly inside the `else` branch, formatted without extra braces
by convention.

In `let PATTERN = expr else { ... };` (let-else), the `else` block runs
only when the pattern fails to match, and that block is required to
diverge — it must `return`, `break`, `continue`, or `panic!`, since control
flow cannot continue past it without the pattern's bindings having been
established.

## Basic usage example

```
if x > 0 {
    println!("positive");
} else {
    println!("non-positive"); // <- `else` runs when the `if` condition is false
}
```

## Embedded Rust Notes

**Full support.** No `std` dependency — behaves identically in `#![no_std]`.

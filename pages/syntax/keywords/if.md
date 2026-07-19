---
title: "if"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language, Pattern Matching]
related_syntax: [else, match]
see_also: [else, match]
---

## Explanation

`if` evaluates a boolean condition and branches accordingly:

```
if x > 0 {
    println!("positive");
}
```

The condition must be a `bool` — Rust has no implicit truthiness
conversion from integers, pointers, or `Option`, unlike C or Python.
Parentheses around the condition are not required (and are conventionally
omitted); the braces around each branch's body are mandatory, even for a
single statement.

Critically, `if` is an **expression**, not just a statement: an
`if`/`else` chain where every branch is a single expression (no trailing
semicolon) produces a value, and all branches must produce the same type:

```
let msg = if x > 0 { "positive" } else { "non-positive" };
```

An `if` with no matching `else` always has type `()`, since the "value"
when the condition is false must exist and match the other branch's type.

`if let PATTERN = expr { ... }` is a distinct form — see the `let`-else
family under pattern matching — that matches a single pattern instead of
testing a `bool`.

## Basic usage example

```
if x > 0 { // <- `if` branches on a boolean condition
    println!("positive");
}
```

## Embedded Rust Notes

**Full support.** No dependency on `std`. Extremely common in embedded
code for polling a hardware flag/register bit before proceeding.

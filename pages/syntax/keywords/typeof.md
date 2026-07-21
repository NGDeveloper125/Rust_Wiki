---
title: "typeof"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`typeof` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
It's held for a hypothetical `typeof(expr)` operator that would give you
the compile-time type of an expression, usable itself as a type —
similar to C++11's `decltype`, or to what `typeof` already means in C
and GNU C extensions.

Rust hasn't needed this in practice because type inference already
covers most of what people reach for `typeof` to do in other languages.
Writing `let x: _ = expr` (or simply `let x = expr`) lets the compiler
infer `x`'s type from `expr` without ever having to name it; generic
functions infer their type parameters from arguments the same way. Where
C++ needs `decltype(expr)` to write a type that depends on another
expression's type, Rust's inference and generics typically make naming
that type unnecessary in the first place. This is likely why `typeof`
has stayed unclaimed since 2015 with no concrete proposal attached to
it — the gap it would fill is already mostly closed by other means.

Using `typeof` as an ordinary identifier is a compile error today. The
raw-identifier form `r#typeof` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### The `typeof` reservation error and raw-identifier escape hatch

```
let typeof = 5;     // error: expected identifier, found reserved keyword `typeof`
let r#typeof = 5;   // ok: the raw-identifier form escapes the reservation
```

## Explanation (Embedded)

No real difference here: `typeof`'s reservation is a lexer-level fact,
checked before any target or runtime is even in the picture, so it's
exactly as unclaimed and exactly as inert under `#![no_std]` as in hosted
Rust. There's no embedded-specific angle to add — no HAL, register, or
interrupt handler changes what this keyword does, because it doesn't do
anything yet in either context.

## Usage examples (Embedded)

### The reservation error is identical under `#![no_std]`

```
#![no_std]

// let typeof = read_register(); // error: expected identifier, found reserved keyword `typeof`
let r#typeof = 0u32; // ok: the raw-identifier form escapes the reservation, same as hosted Rust
```

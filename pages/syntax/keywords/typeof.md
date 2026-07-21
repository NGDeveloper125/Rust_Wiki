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

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

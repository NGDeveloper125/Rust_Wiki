---
title: ":"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Traits, Generics]
related_syntax: [let, fn]
see_also: []
---

## Explanation

`:` introduces a type or constraint annotation. Its exact meaning depends
entirely on position:

- **Variable/parameter type:** `let x: i32 = 5;`, `fn f(x: i32)`
- **Struct field initializer:** `Point { x: 1, y: 2 }`
- **Trait bound:** `fn f<T: Clone>(x: T)` — "`T` must implement `Clone`"
- **Loop label:** `'outer: loop { ... }`
- **Match-arm-like key/value pairs** in some macros

`:` is not an operator and has no overloadable meaning — it's pure
grammar, marking "what follows describes/constrains what came before."
Compare with `::`, a completely different token (path separator) that
happens to look like two of these stacked, but is lexed as its own single
token, not as two colons.

## Embedded Rust Notes

**Full support.** Pure grammar — no `std` dependency.

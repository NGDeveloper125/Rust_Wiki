---
title: "->"
kind: punctuation
embedded_support: full
groups: [Basics, "Functions & Closures"]
related_concepts: [Functions, Closures & capturing]
related_syntax: [fn, "|...| closures"]
see_also: [fn]
---

## Explanation

`->` introduces a function's or closure's return type:

```
fn add(a: i32, b: i32) -> i32 { a + b }
let add = |a: i32, b: i32| -> i32 { a + b };
```

On a closure, `-> Type` is optional and, unlike on `fn`, can usually be
omitted entirely and inferred from the body — it's only required when the
body is ambiguous (e.g. a block whose final expression's type the
compiler can't pin down from a single call site) or when you want to
force a specific type.

`->` is unrelated to `=>` (used in match arms) despite looking similar —
mixing them up is a common typo for newcomers coming from languages where
lambda syntax uses `=>`.

`->` also appears in trait-bound position for `Fn`/`FnMut`/`FnOnce`
trait bounds spelled out explicitly, e.g. `where F: Fn(i32) -> i32`, and
in a bare function-pointer type, `fn(i32) -> i32`.

## Basic usage example

```
fn add(a: i32, b: i32) -> i32 { a + b }
//                      ^^ `->` introduces the return type, `i32`
```

## Embedded Rust Notes

**Full support.** Pure grammar — no `std` dependency.

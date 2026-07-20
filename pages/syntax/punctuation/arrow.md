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

## Best practices & deeper information

### Scenario: Writing generic code

`->` appears in the `Fn`-family trait bound itself, not just on the
generic function that takes the closure — spelling out the bound is how
a generic function declares exactly what shape of closure it accepts.

```
fn apply_twice<F>(x: i32, f: F) -> i32
where
    F: Fn(i32) -> i32, // <- `->` here is part of the trait bound's signature
{
    f(f(x))
}

apply_twice(3, |n| n * 2); // closure inferred to match Fn(i32) -> i32
```

**Why this way:** writing the bound as `Fn(i32) -> i32` rather than a
generic `F: Fn`-with-associated-types spelling is the idiomatic sugar the
language provides specifically for closure/fn-pointer bounds, per the
[Rust Reference on `Fn` traits](https://doc.rust-lang.org/reference/types/closure.html).

### Scenario: Designing a public API

Return-type clarity at the call site is one of the cheapest readability
wins available — `->` is where that type lives, and `impl Trait` in
return position lets an API commit to a trait without exposing the
concrete type.

```
pub fn config_keys() -> impl Iterator<Item = &'static str> {
    // <- `-> impl Iterator<...>` promises "some iterator", nothing more
    ["host", "port", "timeout"].into_iter()
}
```

**Why this way:** returning `impl Trait` rather than a concrete iterator
type (or a boxed trait object) keeps the return type static-dispatched
and zero-cost while hiding an implementation detail the caller shouldn't
depend on. The
[API Guidelines' C-NEWTYPE-HIDE](https://rust-lang.github.io/api-guidelines/future-proofing.html)
discusses the tradeoff against a newtype wrapper — `impl Trait` is the
lighter option when callers won't need extra bounds like `Debug` on the
returned type.

## Embedded Rust Notes

**Full support.** Pure grammar — no `std` dependency.

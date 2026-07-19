---
title: "fn"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Functions, Closures & capturing, Higher-order functions]
related_syntax: ["->", "|...| closures"]
see_also: ["->"]
---

## Explanation

`fn` declares a function:

```
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Every parameter must have an explicit type; unlike closures, `fn`
parameter and return types are never inferred from usage. Omitting the
`-> Type` return-type clause means the function returns `()` (unit). The
final expression in the body (no trailing semicolon) is the return value;
`return` is only needed for an early return.

`fn` also names a distinct family of **function pointer types** — a bare
`fn(i32, i32) -> i32` is a type you can hold in a variable, distinct from
the closure traits (`Fn`/`FnMut`/`FnOnce`). A function item defined with
`fn` can always be coerced to this function-pointer type as long as it
captures nothing.

`fn` can appear standalone (free functions), inside an `impl` block
(associated functions/methods, with or without a `self` receiver), inside
a `trait` (a method signature, optionally with a default body), and
nested inside another function body (an inner function — which, notably,
cannot capture variables from its enclosing scope; only closures can).

## Embedded Rust Notes

**Full support.** Free functions, methods, and function pointers all work
identically in `#![no_std]`. Interrupt handlers are ordinary `fn`s marked
with a target-specific attribute (e.g. `#[interrupt]` from a HAL crate),
not special syntax.

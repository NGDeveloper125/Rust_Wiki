---
title: "!"
kind: operator
embedded_support: full
groups: [Basics, "Macros & Metaprogramming"]
related_concepts: [Operator overloading]
related_syntax: ["!="]
see_also: []
---

## Explanation

As a prefix operator, `!` is logical NOT on `bool` and bitwise complement
on integers, overloadable via `std::ops::Not`:

```
let not_done = !done;      // bool: negation
let flipped = !0b1010u8;   // integer: bitwise complement -> 0b11110101
```

Immediately after an identifier or path with no space, `!` instead marks a
**macro invocation** — a completely unrelated meaning:

```
println!("hi");
vec![1, 2, 3];
```

`ident!` is not the `Not` trait applied to `ident`; it's the macro-call
syntax, and the parser distinguishes it purely by the identifier
immediately preceding the `!` with no space or operator between them.

`!` alone (no operand, in type position) is also the **never type**
(`fn diverges() -> !`) — the type of an expression that never produces a
value, such as `return`, `break`, `panic!()`, or an infinite `loop` with
no `break`.

## Basic usage example

```
let done = false;
let not_done = !done; // <- `!` negates the bool
```

## Embedded Rust Notes

**Full support.** `Not` lives in `core::ops`; macro invocation and the
never type `!` are core grammar — none of this depends on `std`. Note
that `panic!()` itself behaves differently under `#![no_std]` (see the
[panic! macro](../macros/panic.md) and
[Panic & unwinding](../../concepts/panic-and-unwinding.md) pages) — it
still expands via this same `!` syntax, but requires a `#[panic_handler]`
function since there's no default one without `std`.

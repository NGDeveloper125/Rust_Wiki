---
title: "( )"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Functions]
related_syntax: [","]
see_also: [","]
---

## Explanation

`( )` serves several distinct roles depending on context:

- **Grouping:** `(a + b) * c` — overrides normal precedence, same as
  in arithmetic notation generally.
- **Tuple expression/type:** `(1, "a", true)` is a 3-tuple value;
  `(i32, &str, bool)` is its type. `()` with nothing inside is the
  **unit** value/type — Rust's "no meaningful value" type, distinct from
  `void` in that it's a real, first-class, zero-sized type you can bind,
  pass, and return.
- **Single-element tuple:** `(x,)` — the trailing comma is mandatory (see
  [`,`](comma.md)); without it, `(x)` is just `x` grouped, not a tuple.
- **Function call / tuple-struct or enum-variant construction:**
  `f(a, b)`, `Point(1, 2)`, `Some(x)`.

Which meaning applies is determined entirely by what (if anything)
immediately precedes the `(` — an identifier/path means a call or
construction; nothing (or an operator) means grouping or a tuple.

## Basic usage example

```rust
fn add(a: i32, b: i32) -> i32 { a + b } // <- `( )` groups the parameter list
let sum = add(1, 2); // <- `( )` here is the call, passing the arguments
let pair = (1, "a"); // <- `( )` here builds a tuple value
```

## Embedded Rust Notes

**Full support.** Grouping, tuples, and calls are core grammar — no `std`
dependency. The unit type `()` in particular is exactly as zero-cost on
an embedded target as anywhere else.

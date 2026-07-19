---
title: "const"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Const generics, "Rust Philosophy & Design Principles"]
related_syntax: [static, let]
see_also: [static]
---

## Explanation

`const` declares a compile-time constant:

```
const MAX_POINTS: u32 = 100_000;
```

A `const` must have an explicit type annotation (unlike `let`, type
inference alone is not enough) and its value must be computable entirely
at compile time — the initializer runs through Rust's const evaluator, not
at runtime. There is no fixed memory address for a `const`: every place it
is used, the compiler is free to inline its value directly, the same way
it would inline a literal.

`const` items can be declared at module scope, inside a function body,
inside a `trait`/`impl` block (an *associated const*), and inside a
`struct`/`enum` definition's generic parameter list — an entirely
different use, introducing a **const generic** parameter
(`struct Buffer<const N: usize>`), which parameterizes a type by a value
rather than by another type.

Naming convention is `SCREAMING_SNAKE_CASE`. `const` bindings are always
implicitly immutable — `const mut` does not exist.

## Basic usage example

```
const MAX_POINTS: u32 = 100_000; // <- `const` declares a compile-time constant
```

## Embedded Rust Notes

**Full support.** `const` is especially valuable in embedded code:
register addresses, buffer sizes, and lookup tables computed entirely at
compile time cost zero flash/RAM beyond the value itself, with no runtime
initialization needed.

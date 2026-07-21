---
title: "const"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Const generics, "Zero-cost abstractions"]
related_syntax: [static, let]
see_also: [static]
---

## Explanation

`const` declares a compile-time constant, as in
`const MAX_POINTS: u32 = 100_000;`.

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

## Best practices & deeper information

### Scenario: Numeric computation

A buffer size that's a fixed multiple of a record size is exactly the
kind of arithmetic `const` is for — it's computed once, at compile time,
instead of being recomputed (or hand-typed as a literal) wherever it's
needed.

```
const MAX_RETRY_COUNT: u8 = 5;
const BUFFER_CAPACITY: usize = 64 * 4; // <- `const` requires a value computable entirely at compile time

fn should_retry(attempt: u8) -> bool {
    attempt < MAX_RETRY_COUNT
}
```

**Why this way:** the initializer expression runs through the compiler's
const evaluator rather than at runtime, so `BUFFER_CAPACITY` costs nothing
beyond the final value — the
[Reference's constant items](https://doc.rust-lang.org/reference/items/constant-items.html)
page confirms the initializer must itself be a const expression, which is
exactly what makes this guarantee possible.

### Scenario: Designing a public API

A library that wants callers to know its default timeout, and to be able
to reference it by name instead of a magic number, exposes the value as a
public `const`.

```
/// Default timeout applied when a caller doesn't specify one.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30; // <- `const`, public, SCREAMING_SNAKE_CASE per convention

pub fn connect_with_default_timeout() {
    connect(DEFAULT_TIMEOUT_SECS)
}

fn connect(_timeout_secs: u64) {}
```

**Why this way:** `SCREAMING_SNAKE_CASE` for constants is codified in the
[API Guidelines' casing conventions](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case),
and publishing the constant lets downstream code refer to
`DEFAULT_TIMEOUT_SECS` by name instead of duplicating the literal `30` at
every call site.

## Embedded Rust Notes

**Full support.** `const` is especially valuable in embedded code:
register addresses, buffer sizes, and lookup tables computed entirely at
compile time cost zero flash/RAM beyond the value itself, with no runtime
initialization needed.

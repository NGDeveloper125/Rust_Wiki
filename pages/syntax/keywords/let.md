---
title: "let"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership, Immutability by default, Pattern Matching]
related_syntax: [mut, const, static, if, else]
see_also: [mut, const]
---

## Explanation

`let` introduces a new variable binding in the current scope. It binds a
name to the value produced by an expression:

```
let x = 5;
```

This is a declaration, not an assignment in the C sense — `let` always
creates a *new* binding, even if a variable of the same name already
exists. Using `let` again with a name already in scope **shadows** the
previous binding rather than mutating it; the old value still exists (and
may still be borrowed elsewhere) until it goes out of scope or is dropped.

Bindings introduced by `let` are immutable unless the pattern includes
`mut` (see the [`mut`](mut.md) page) — `let` itself does not imply
mutability. A `let` can:

- carry an explicit type annotation: `let x: i32 = 5;`
- destructure a pattern: `let (a, b) = pair;`, `let Point { x, y } = p;`
- be refutable when paired with `else` (`let Some(x) = opt else { return };`)
  — the pattern must match or the `else` block runs and must diverge
- appear with no initializer at all (`let x;`), deferring assignment,
  as long as the compiler can prove it's assigned before first use

`let` is a **statement**, not an expression — it has no value of its own
and cannot be used where an expression is required.

## Embedded Rust Notes

**Full support.** `let` is core language grammar with no dependency on
`std` — bindings work identically in `#![no_std]` firmware, on the stack,
exactly as on a hosted target.

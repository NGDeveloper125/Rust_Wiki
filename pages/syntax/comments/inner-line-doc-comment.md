---
title: "//! (inner line doc comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [line-comment, outer-line-doc-comment]
see_also: [outer-line-doc-comment]
---

## Explanation

`//!` documents the **enclosing** item (the module or crate it appears
inside) rather than the item that follows it — the opposite direction
from `///`. It's typically placed at the very top of a file, documenting
the module/crate as a whole:

```
//! This module implements the parser's tokenizer.
```

Because it documents its *container*, `//!` is the only doc-comment form
that can appear with nothing syntactically after it at all (e.g. at the
top of `lib.rs`/`main.rs`, documenting the whole crate).

## Basic usage example

```
//! <- this doc comment documents the enclosing module/crate, not an item below it
//! This module implements the parser's tokenizer.

fn tokenize() {}
```

**Restriction:** `//!` documents whatever it's *inside*, so it's normally
placed at the very top of a file (`lib.rs`/`main.rs`, or a module file) —
placing it deep inside a function body would still compile but wouldn't
mean what you expect.

## Best practices & deeper information

### Scenario: Documenting an API

The idiomatic place for `//!` is the very top of `lib.rs` (or a module
file), giving the crate/module a landing-page summary before any item
docs are reached.

```
//! # my_crate
//!
//! A small library for parsing human-readable duration strings like
//! `"5s"` or `"10m"` into a [`std::time::Duration`].
// <- this `//!` block documents the crate itself; `cargo doc` renders it
//    as the top-level page for my_crate

pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    todo!()
}
```

**Why this way:** `cargo doc` uses the crate-root `//!` block as the
front page of the generated documentation site — it's the first (and
sometimes only) thing a new user of the crate reads, per the
[rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#documenting-components).

### Scenario: Designing a public API

For a module with real invariants or safety expectations, `//!` is the
place to state them once, up front, instead of repeating the same caveat
on every item inside.

```
//! Fixed-capacity ring buffer.
//!
//! All indices are taken modulo the buffer's capacity; capacity must be
//! a power of two, enforced by [`RingBuffer::new`].
// <- states the module-wide invariant once, instead of on every method

pub struct RingBuffer<T> { /* ... */ }
```

**Why this way:** stating an invariant at the module level (`//!`) rather
than duplicating it across every method's `///` comment keeps the
guarantee in exactly one place — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html)
favor documenting an invariant where it's enforced, not everywhere it's
relied upon.

## Embedded Rust Notes

**Full support.** Same as [`///`](outer-line-doc-comment.md) — no `std`
dependency, same host-vs-target doc test caveat.

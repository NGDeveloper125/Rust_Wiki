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

```rust
//! <- this doc comment documents the enclosing module/crate, not an item below it
//! This module implements the parser's tokenizer.

fn tokenize() {}
```

**Restriction:** `//!` documents whatever it's *inside*, so it's normally
placed at the very top of a file (`lib.rs`/`main.rs`, or a module file) —
placing it deep inside a function body would still compile but wouldn't
mean what you expect.

## Embedded Rust Notes

**Full support.** Same as [`///`](outer-line-doc-comment.md) — no `std`
dependency, same host-vs-target doc test caveat.

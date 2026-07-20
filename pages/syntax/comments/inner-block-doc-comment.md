---
title: "/*! */ (inner block doc comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [block-comment, inner-line-doc-comment]
see_also: [inner-line-doc-comment]
---

## Explanation

`/*! ... */` is the block-comment equivalent of `//!` — it documents the
enclosing module/crate rather than a following item, as in
`/*! This module implements the parser's tokenizer. */`.

As with `/** */` vs `///`, this form is rarely used in idiomatic Rust —
`//!` is the conventional choice — but both are equivalent to the
compiler.

## Basic usage example

```
/*! This module implements the parser's tokenizer. */
// ^ the `/*!` above documents this whole module, not `tokenize` below

fn tokenize() {}
```

**Restriction:** same as `//!` — it documents its container, so it's
normally placed at the top of the file it applies to. In practice,
prefer `//!` (see [`//!`](inner-line-doc-comment.md)); this form is rare.

## Best practices & deeper information

### Scenario: Documenting an API

`/*! */` documents its enclosing module/crate exactly like `//!` — again,
purely a stylistic choice, and `//!` is what idiomatic code uses.

```
/*! # my_crate

A small library for parsing duration strings. */
// <- `/*! */` above documents the crate itself, same as `//!` would

pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    todo!()
}

pub struct ParseError;
```

**Why this way:** see [`//!`](inner-line-doc-comment.md) for the full
treatment (crate-root landing page, stating module invariants once) —
everything there applies here unchanged. `/*! */` is rare enough in real
codebases that introducing it fresh mostly just surprises readers
expecting `//!`.

## Embedded Rust Notes

**Full support.** Same as [`//!`](inner-line-doc-comment.md) — no `std`
dependency.

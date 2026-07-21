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

## Usage examples

### Documenting the enclosing module

```
/*! This module implements the parser's tokenizer. */
// ^ the `/*!` above documents this whole module, not `tokenize` below

fn tokenize() {}
```

**Restriction:** same as `//!` — it documents its container, so it's
normally placed at the top of the file it applies to. In practice,
prefer `//!` (see [`//!`](inner-line-doc-comment.md)); this form is rare.

### Documenting an API

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

See [`//!`](inner-line-doc-comment.md) for the full
treatment (crate-root landing page, stating module invariants once) —
everything there applies here unchanged. `/*! */` is rare enough in real
codebases that introducing it fresh mostly just surprises readers
expecting `//!`.

## Explanation (Embedded)

`/*! ... */` documents its enclosing module/crate identically under
`#![no_std]` — doc generation doesn't touch `std` at all. See
[`//!`](inner-line-doc-comment.md) for the fuller embedded-specific
discussion (crate-root docs stating hardware assumptions, the
host-vs-target doc test caveat); everything there applies to this block
form unchanged. As with the hosted-Rust version, `//!` is still the
idiomatic choice in embedded crates too — this form is rare.

## Usage examples (Embedded)

### Documenting a HAL crate's assumptions

```
/*! Peripheral access crate for the XYZ-100 microcontroller.

Assumes a 16 MHz external crystal; call `clocks::init()` before touching
any other peripheral, or reads return unspecified values. */
// <- crate-root doc states the one hardware assumption every user must know

pub mod clocks;
pub mod gpio;
```

Stating the crystal-frequency assumption once at the crate root, in
whichever doc-comment form the crate already favors, saves every
peripheral module from repeating it.

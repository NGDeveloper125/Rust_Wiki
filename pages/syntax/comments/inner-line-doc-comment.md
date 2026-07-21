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
the module/crate as a whole, as in `//! This module implements the
parser's tokenizer.`.

Because it documents its *container*, `//!` (along with its block form
[`/*! */`](inner-block-doc-comment.md)) can appear with nothing
syntactically after it at all (e.g. at the top of `lib.rs`/`main.rs`,
documenting the whole crate) — something no *outer* doc-comment form
can do.

## Usage examples

### Documenting the enclosing module

```
//! <- this doc comment documents the enclosing module/crate, not an item below it
//! This module implements the parser's tokenizer.

fn tokenize() {}
```

**Restriction:** `//!` documents whatever it's *inside*, so it's normally
placed at the very top of a file (`lib.rs`/`main.rs`, or a module file) —
placing it deep inside a function body would still compile but wouldn't
mean what you expect.

### Documenting an API

The idiomatic place for `//!` is the very top of `lib.rs` (or a module
file), giving the crate/module a landing-page summary before any item
docs are reached.

```
//! # my_crate
//!
//! A small library for parsing human-readable duration strings like
//! `"5s"` or `"10m"` into whole seconds.
// <- this `//!` block documents the crate itself; `cargo doc` renders it
//    as the top-level page for my_crate

pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    todo!()
}

pub struct ParseError;
```

`cargo doc` uses the crate-root `//!` block as the
front page of the generated documentation site — it's the first (and
sometimes only) thing a new user of the crate reads, per the
[rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html#documenting-components).

### Designing a public API

For a module with real invariants or safety expectations, `//!` is the
place to state them once, up front, instead of repeating the same caveat
on every item inside.

```
//! Fixed-capacity ring buffer.
//!
//! All indices are taken modulo the buffer's capacity; capacity must be
//! a power of two, enforced by the constructor.
// <- states the module-wide invariant once, instead of on every method

pub struct RingBuffer<T> {
    items: Vec<T>,
    head: usize,
}
```

Stating an invariant at the module level (`//!`) rather
than duplicating it across every method's `///` comment keeps the
guarantee in exactly one place — when the invariant changes, there's one
paragraph to update instead of a scattered set of near-copies that will
inevitably drift apart.

## Explanation (Embedded)

`//!` works identically under `#![no_std]` — doc generation is a
compile-time, host-side process that never touches the target. In an
embedded crate, the crate-root `//!` block is the natural place to state
the assumptions a HAL or driver crate depends on (clock source and speed,
which pins are safe to touch before `clocks::init()`, whether an API is
interrupt-safe) — the kind of fact that would otherwise be scattered
across every module that relies on it.

## Usage examples (Embedded)

### Stating a crate's hardware assumptions once

```
//! Peripheral access crate for the XYZ-100 microcontroller.
//!
//! Assumes a 16 MHz external crystal; call `clocks::init()` before
//! touching any other peripheral, or reads return unspecified values.
// <- crate-root `//!` states the one assumption every user of this crate must know

pub mod clocks;
pub mod gpio;
```

**Restriction:** same as hosted Rust — `//!` documents whatever module it
appears inside, so this block belongs at the top of `lib.rs` (or the
module it describes), not scattered near individual peripheral functions.

### Documenting a module's interrupt-safety contract

```
//! SPI driver for the onboard flash chip.
//!
//! None of these functions are interrupt-safe: calling `flash::read()`
//! from an interrupt handler while the main loop holds the bus will
//! corrupt the transfer. Disable interrupts around any call made outside
//! `main`, or route flash access through a critical section.
// <- states an invariant that's easy to violate and hard to debug once violated

pub fn read(address: u32, buf: &mut [u8]) {
    // ...
}
```

Doc tests inside a `//!` block still compile and run on the **host**
toolchain, not the target microcontroller — fine for pure logic, but any
example that touches real registers needs `no_run` (compiles, doesn't
execute) since the host has no XYZ-100 peripherals to run it against.

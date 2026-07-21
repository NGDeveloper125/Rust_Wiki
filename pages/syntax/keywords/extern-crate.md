---
title: "extern crate"
kind: keyword
embedded_support: partial
groups: ["Modules & Visibility", "Modules, Crates & Visibility", "Memory & Unsafe"]
related_concepts: [Crates]
related_syntax: [extern, mod, use]
see_also: [extern]
---

## Explanation

Before the 2018 edition, every crate a project depended on had to be
declared twice: once in `Cargo.toml` under `[dependencies]`, and again in
source with an `extern crate` item — `extern crate serde;` — to actually
bring that crate's root into scope as a path segment usable from the
current crate.

The 2018 edition made this automatic: any crate listed in `Cargo.toml` is
available as a path root (`serde::Deserialize`, etc.) and importable with
`use` without ever writing `extern crate`. In practice this makes
`extern crate` almost entirely unnecessary today — it mostly survives in
codebases written before 2018, and even that code keeps compiling after an
edition migration without removing it, since an explicit `extern crate`
line is still legal, just redundant.

One case where `extern crate` is still written deliberately today: a
`#![no_std]` crate that wants the `alloc` crate (heap-allocating types
like `Vec`/`String`/`Box`, without the rest of `std`) needs
`extern crate alloc;` at its crate root to bring `alloc` into scope. Unlike
an ordinary `Cargo.toml` dependency, `alloc` (like `core`) isn't
implicitly linked by the 2018-edition mechanism — it's part of the
standard distribution but has to be opted into explicitly this way.

See [`extern`](extern.md) for the unrelated FFI/ABI use of the plain
`extern` keyword (`extern "C" fn`, `extern` blocks) — this page covers
only the crate-declaration form, `extern crate`.

## Usage examples

### Opting a no_std crate into alloc

```
#![no_std]

extern crate alloc; // <- still required today: opts a `#![no_std]` crate into the `alloc` crate

use alloc::vec::Vec;
```

### Designing a public API

A `#![no_std]` library crate offers a `Vec`-returning public function, so
it needs `alloc` explicitly linked before it can use `alloc::vec::Vec` at
all.

```
#![no_std]

extern crate alloc; // <- opts this crate into `alloc`; not automatic even in modern editions

use alloc::vec::Vec;

pub fn duplicate_last(values: &[u32]) -> Vec<u32> { // <- public API returning an `alloc` type
    let mut out = Vec::new();
    out.extend_from_slice(values);
    if let Some(&last) = values.last() {
        out.push(last);
    }
    out
}
```

`alloc` sits between `core` (always available,
allocation-free) and `std` (requires an OS), so a `#![no_std]` crate that
still wants heap-allocating types opts into it explicitly with
`extern crate alloc;` rather than gaining it implicitly — the requirement
the
[Rust Reference's extern crate declarations](https://doc.rust-lang.org/reference/items/extern-crates.html)
documents, with `alloc`'s own scope described in the
[`alloc` crate docs](https://doc.rust-lang.org/alloc/).

## Explanation (Embedded)

**Partial support, and this is squarely where the keyword still earns
its keep.** `extern crate` itself is compile-time-only and costs nothing
either way, but essentially its only living use case today is
embedded/`#![no_std]`: bringing the `alloc` crate into scope with
`extern crate alloc;` so a `#![no_std]` firmware crate can use
heap-allocating types (`Vec`, `String`, `Box`) without pulling in the
rest of `std`, which would require an OS this target doesn't have.
`core` needs no equivalent declaration — it's implicitly available under
`#![no_std]` the same way `std` is implicitly available in a hosted
crate — but `alloc` sits in between and has to be opted into explicitly
this way, interacting directly with `#![no_std]`'s stripped-down
implicit-prelude wiring. The caveat that comes with it: `alloc` alone
supplies the *types*, not an allocator — a `#![no_std]` binary using it
also needs a `#[global_allocator]` (commonly `embedded-alloc` or a
similar bare-metal allocator) wired up, or every allocation panics at
runtime with nothing to satisfy it. Many embedded codebases sidestep
both the `extern crate alloc;` declaration and the global-allocator
requirement entirely by reaching for `heapless`'s fixed-capacity
collections (`heapless::Vec<T, N>`, `heapless::String<N>`) instead,
which need no heap and no allocator at all — the idiomatic embedded
substitute whenever a fixed upper bound on size is acceptable, which it
very often is in firmware.

## Usage examples (Embedded)

### Opting into `alloc` alongside a global allocator

```
#![no_std]
#![no_main]

extern crate alloc; // <- still required: opts this `#![no_std]` crate into the `alloc` crate

use alloc::vec::Vec;
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty(); // <- required alongside `extern crate alloc;`: alloc supplies types, not memory

fn collect_samples(readings: &[u16]) -> Vec<u32> {
    readings.iter().map(|&r| r as u32).collect() // <- `Vec` only usable because both lines above are present
}
```

### The `heapless` substitute — no `extern crate alloc;`, no global allocator

```
#![no_std]

use heapless::Vec; // <- fixed-capacity Vec: no `extern crate alloc;` and no `#[global_allocator]` needed

fn collect_samples(readings: &[u16]) -> Vec<u32, 32> { // <- capacity fixed at compile time via the const generic
    readings.iter().map(|&r| r as u32).collect()
}
```

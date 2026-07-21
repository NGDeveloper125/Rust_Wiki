---
title: "extern crate"
kind: keyword
embedded_support: partial
groups: ["Modules, Crates & Visibility", "Memory & Unsafe"]
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

## Basic usage example

```
#![no_std]

extern crate alloc; // <- still required today: opts a `#![no_std]` crate into the `alloc` crate

use alloc::vec::Vec;
```

## Best practices & deeper information

### Scenario: Designing a public API

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

**Why this way:** `alloc` sits between `core` (always available,
allocation-free) and `std` (requires an OS), so a `#![no_std]` crate that
still wants heap-allocating types opts into it explicitly with
`extern crate alloc;` rather than gaining it implicitly — the requirement
the
[Rust Reference's extern crate declarations](https://doc.rust-lang.org/reference/items/extern-crates.html)
documents, with `alloc`'s own scope described in the
[`alloc` crate docs](https://doc.rust-lang.org/alloc/).

## Embedded Rust Notes

**Partial support — this is where the keyword still earns its keep.**
`extern crate` itself is compile-time-only and costs nothing either way,
but its remaining relevance today is almost entirely an embedded/`no_std`
concern: `extern crate alloc;` is the standard way a `#![no_std]` firmware
crate opts into heap-allocating collections, typically paired with a
`#[global_allocator]`. `core` needs no equivalent declaration — it's
implicitly available even under `#![no_std]`.

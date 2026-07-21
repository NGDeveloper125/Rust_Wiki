---
title: "#[crate_type = \"...\"] / #[crate_name = \"...\"]"
kind: attribute
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["FFI (foreign function interface)"]
related_syntax: []
see_also: []
---

## Explanation

Both attributes are written as inner attributes (`#![...]`) at the top of
a crate root, and both set something today more commonly configured from
`Cargo.toml` instead. They're covered together because they serve the
same purpose — telling a standalone `rustc` invocation something Cargo
would otherwise supply — and both are considerably rarer to see written
by hand than they were before Cargo existed.

`#![crate_type = "..."]` selects what kind of artifact the compiler
produces: `"bin"` for an executable, `"lib"` for an ordinary Rust library,
`"dylib"`/`"staticlib"`/`"cdylib"` for various forms of dynamic or static
library (the latter two are the forms consumable from C or another
language), and `"rlib"` for Rust's own intermediate library format.
Multiple values can be given to produce several artifact kinds from one
compilation. In a Cargo project, this is set via the `[lib]` table's
`crate-type` key in `Cargo.toml` instead, which is the idiomatic place for
it — `#![crate_type = "..."]` matters mainly when invoking `rustc`
directly on a single file, without Cargo in the picture at all.

`#![crate_name = "..."]` sets the compiled crate's name explicitly from
within the source file, overriding the name Cargo would otherwise derive
from the package's `name` in `Cargo.toml` (or, without Cargo, from the
source file's own name). Like `crate_type`, this is mostly relevant to
bare `rustc` invocations, documentation generation setups, or unusual
build pipelines that compile a file without a surrounding Cargo project;
inside an ordinary Cargo-managed crate, the package name in `Cargo.toml`
is the conventional place to control this.

## Basic usage example

```
#![crate_type = "lib"] // <- rustc-only: produces an rlib without a surrounding Cargo project
#![crate_name = "telemetry_core"] // <- names the crate explicitly rather than deriving it from the filename

pub fn version() -> &'static str {
    "0.1.0"
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A small internal tool is compiled directly with `rustc` as part of a
custom build pipeline, with no `Cargo.toml` in the picture — the crate
name and output kind have to come from somewhere, so they're declared in
the source itself.

```
#![crate_type = "bin"] // <- produces an executable; no Cargo.toml present to say so otherwise
#![crate_name = "log_rotator"]

fn main() {
    println!("rotating logs");
}
```

**Why this way:** in an ordinary Cargo-managed project, `[lib]
crate-type` and the package `name` in `Cargo.toml` are the idiomatic,
single source of truth for both of these, and duplicating them into
`#![crate_type]`/`#![crate_name]` as well is generally avoided since the
two can silently disagree; the
[rustc book](https://doc.rust-lang.org/rustc/command-line-arguments.html#--crate-type-a-list-of-types-of-crates-for-the-compiler-to-emit)
documents these attributes as the source-level equivalent of the
`--crate-type`/`--crate-name` command-line flags used when Cargo isn't
involved at all.

## Embedded Rust Notes

**Full support**, with a caveat worth stating plainly: embedded firmware
projects almost always need `#![crate_type = "bin"]` or `"staticlib"`
implicitly satisfied through Cargo's own `[[bin]]`/`[lib]` configuration
and target-specific linker setup, rather than through this attribute
directly. It behaves identically in `#![no_std]` in the rare cases it is
used, but idiomatic embedded projects lean on Cargo and a `.cargo/config.toml`
target/linker setup instead of writing this attribute by hand.

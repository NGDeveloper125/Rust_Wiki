---
title: "Cargo & Cargo.toml"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project", "Unique to Rust"]
related_syntax: []
see_also: ["Crates", "Workspaces", "Dependency management & semver"]
---

## Explanation

Cargo is Rust's build tool and package manager — the single command-line
entry point (`cargo build`, `cargo test`, `cargo run`, `cargo publish`)
that reads a crate's `Cargo.toml` manifest, resolves its dependencies,
invokes `rustc` with the right flags, and produces the resulting build
artifacts. Every [crate](crates.md) has exactly one `Cargo.toml`
describing its name, version, edition, dependencies, and any optional
feature flags.

A `Cargo.toml` is organized into a small number of tables: `[package]`
holds the crate's own metadata (name, version, edition); `[dependencies]`,
`[dev-dependencies]`, and `[build-dependencies]` list what it depends on
(see
[dependency management & semver](dependency-management-and-semver.md)
for what goes in those version strings); `[features]` declares optional,
additive capability flags consumers can opt into; and, only at the root
of a multi-crate project, `[workspace]` lists the member crates (see
[Workspaces](workspaces.md)).

Cargo's job goes beyond invoking the compiler: it runs a dependency
resolver that picks one mutually-compatible version of every crate a
project needs and pins the result in `Cargo.lock`; it fetches crates from
crates.io (or another registry); it runs tests, benchmarks, and
documentation generation; and `cargo publish` is the actual mechanism by
which a crate ships to the rest of the ecosystem. Cargo and crates.io
together are frequently cited as one of Rust's biggest advantages over
languages without an integrated, standard build and package tool.

`Cargo.toml` says nothing about how code inside a crate is organized
(that's [modules](modules.md)) or what's visible to callers (that's
[visibility & privacy](visibility-and-privacy.md)) — those are
language-level concerns. Cargo governs the ecosystem and tooling layer
that sits around the language: what a crate is called, what it depends
on, and how it gets built, tested, and shipped.

## Basic usage example

`Cargo.toml` is TOML, not Rust code — this is the file's actual content,
not a Rust snippet.

```
[package]
name = "weather-api"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] } # <- Cargo resolves and downloads this crate
```

## Best practices & deeper information

### Scenario: Documenting an API

A crate that offers optional functionality behind Cargo features should
document those features directly, so users know what
`default-features = false` turns off before hitting a missing-symbol
error instead of after.

```
[package]
name = "weather-api"
version = "0.1.0"
edition = "2024"

[features]
default = ["blocking"]
blocking = []              # <- synchronous client, included by default
async = ["dep:tokio"]      # <- opt-in: only compiled if a consumer enables it

[dependencies]
tokio = { version = "1", features = ["rt"], optional = true } # <- only pulled in by the `async` feature
```

**Why this way:** naming and documenting feature flags explicitly, rather
than leaving consumers to discover them by reading source, is part of
the API Guidelines' checklist for
[conditional compilation via features](https://rust-lang.github.io/api-guidelines/flexibility.html) —
a crate's `Cargo.toml` is as much of its public documentation as its
doc comments.

## Explanation (Embedded)

Cargo's own job — reading a manifest, resolving dependencies, invoking
`rustc` — doesn't change for an embedded target; there is no separate
"embedded Cargo." What's genuinely different is the layer of tooling and
configuration embedded projects add around that same Cargo. A
`.cargo/config.toml` typically pins a default `--target` (a target
triple like `thumbv7em-none-eabihf` for a Cortex-M chip) so `cargo
build`/`cargo run` cross-compile without repeating `--target` on every
invocation, and it can also set a `runner` — a command Cargo hands the
built binary to, such as `probe-rs run` to flash and run it on a physical
debug probe, or `qemu-system-arm` to run it in emulation — so `cargo run`
does something meaningful even though the host machine can't execute
Cortex-M machine code directly. `Cargo.toml` itself also carries more
weight on an embedded target: `[profile.release]` settings that are
merely nice-to-have on a desktop binary — `panic = "abort"` to drop
unwinding machinery entirely, `lto = true` for cross-crate inlining,
`opt-level = "s"` to optimize for code size instead of speed — routinely
decide whether a firmware image fits in the target's flash at all.

## Basic usage example (Embedded)

```
# .cargo/config.toml
[build]
target = "thumbv7em-none-eabihf"   # <- default cross-compilation target triple

[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411CEUx"   # <- flashes and runs the built binary on real hardware
```

## Best practices & deeper information (Embedded)

### Scenario: Targeting a microcontroller from Cargo.toml

A firmware crate needs every `cargo build`/`cargo run` to cross-compile
for its Cortex-M target and, on `cargo run`, actually get onto the
board, without every teammate typing `--target` and a flashing command
by hand.

```
# .cargo/config.toml
[build]
target = "thumbv7em-none-eabihf"        # <- fixes the target triple for this crate

[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411CEUx"  # <- `cargo run` flashes + runs via a debug probe

# Cargo.toml
[package]
name = "blinky"
edition = "2024"

[dependencies]
cortex-m-rt = "0.7"
panic-halt = "0.2"
```

**Why this way:** committing the target and runner to
`.cargo/config.toml` (checked into the repo) rather than each
developer's shell profile means `cargo run` reproduces the same
flash-and-run behavior on every machine, the setup the
[probe-rs documentation](https://probe.rs/docs/tools/cargo-embed/)
itself recommends for a firmware crate.

### Scenario: Shrinking a release build for constrained flash

A Cortex-M0's flash is measured in tens of kilobytes, so a
debug-profile-sized binary — with unwinding tables and speed-optimized
code — routinely doesn't fit; the release profile has to be tuned for
size, not just turned on.

```
# Cargo.toml
[profile.release]
panic = "abort"     # <- drops unwinding machinery entirely; panics just halt
lto = true           # <- cross-crate inlining shrinks the final image
opt-level = "s"      # <- optimize for size rather than speed
codegen-units = 1    # <- trades build time for better size optimization
```

**Why this way:** `panic = "abort"` alone commonly removes several
kilobytes of unwinding tables that a `#![no_std]` binary has no use for,
and the
[Embedded Rust Book's optimization guidance](https://docs.rust-embedded.org/book/)
treats `opt-level = "s"`/`"z"` plus LTO as the standard first move when a
release build doesn't fit in flash.

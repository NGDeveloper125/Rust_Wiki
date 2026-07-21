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

## Embedded Rust Notes

**Full support.** Cargo builds identically regardless of target — there
is no separate "embedded Cargo." The only embedded-specific detail is
specifying a target explicitly, e.g. `cargo build --target
thumbv7em-none-eabihf`, often paired with a `.cargo/config.toml` setting
a default target and runner; a `#![no_std]` crate's `Cargo.toml` looks
like any other, typically with `default-features = false` on
dependencies to opt out of their `std`-dependent code paths.

---
title: "Workspaces"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project"]
related_syntax: []
see_also: ["Crates", "Cargo & Cargo.toml"]
---

## Explanation

A workspace is a set of crates that are built and locked together as one
project, declared by a top-level `Cargo.toml` containing a `[workspace]`
table that lists the member crates. Every crate in a workspace shares one
`Cargo.lock` and one `target/` build directory, so the whole set of
crates always resolves to a single, mutually-compatible version of every
shared dependency — there's no risk of two members of the same project
silently compiling against two different versions of the same library.

The mental model is a folder of sibling [crates](crates.md) — say, a CLI
binary crate and one or two library crates it depends on — that live in
one repository and are developed, tested, and built as a unit, without
any one of them needing to be published to crates.io just so another
member can use it (a workspace member typically depends on another
member with a `path` dependency instead of a version).

Workspaces exist because [modules](modules.md) alone can't give a
project everything a crate boundary provides: independent compilation,
an independent [semver version](dependency-management-and-semver.md),
and a real [privacy](visibility-and-privacy.md) wall between, say, a
thin CLI front-end and the library logic it wraps. Splitting code into
modules only reorganizes names inside one crate; splitting it into
workspace members gives each piece its own crate identity while keeping
the convenience of one repository, one lockfile, and one `cargo build`
invocation for everything.

Cargo commands run workspace-wide by default from the root — `cargo
build`/`cargo test` build or test every member — or can target a single
member with `-p <crate-name>`. Everything else about a member crate,
from its own `[dependencies]` to its own version number, still follows
the same rules as a standalone crate; a workspace changes how crates are
built and locked together, not what a crate is.

## Basic usage example

The workspace root's `Cargo.toml` has no `[package]` section of its own —
just the member list — and each member still has its own ordinary
`Cargo.toml`. This example shows plain TOML/directory-layout content, not
Rust code.

```
# Cargo.toml (workspace root)
[workspace]
members = [
    "app",            # <- binary crate: the CLI front-end
    "weather-api",    # <- library crate: shared logic
]

# app/Cargo.toml
[dependencies]
weather-api = { path = "../weather-api" } # <- workspace member depended on by local path
```

## Best practices & deeper information

### Scenario: Designing a public API

A CLI app that started as a single binary crate grows a reusable core —
an HTTP client plus response parsing — that a second binary, a
background sync daemon, also needs; moving that core into its own
workspace member lets both binaries share it without duplicating code or
publishing anything externally yet.

```
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "weather-cli",    # <- existing binary crate
    "weather-sync",   # <- new binary: a background sync daemon
    "weather-core",   # <- new library crate: logic shared by both binaries
]

# weather-cli/Cargo.toml
[dependencies]
weather-core = { path = "../weather-core" } # <- depends on the shared crate by path, no version needed yet

# weather-sync/Cargo.toml
[dependencies]
weather-core = { path = "../weather-core" } # <- same shared crate, second consumer
```

**Why this way:** splitting reusable logic into its own focused crate
keeps each piece's public API and compile unit small instead of growing
one crate to cover unrelated concerns, which is exactly the
[Rust Design Patterns' "Prefer small crates"](https://rust-unofficial.github.io/patterns/idioms/prefer-small-crates.html)
idiom argues for — a workspace is what makes that split practical without
publishing anything or losing single-repository convenience.

## Embedded Rust Notes

**Full support.** A workspace is purely a Cargo/build-system construct
with no runtime effect, so it works the same when one or more members
target embedded hardware — for example, a workspace containing a
`no_std` firmware crate alongside a `std`-based logic crate that both the
firmware and a desktop simulator depend on. The one embedded-specific
detail is that building a single member for a target still needs `-p`
and `--target` together, e.g. `cargo build --target thumbv7em-none-eabihf
-p firmware`, since other workspace members (like a host-side simulator)
may not build for that target at all.

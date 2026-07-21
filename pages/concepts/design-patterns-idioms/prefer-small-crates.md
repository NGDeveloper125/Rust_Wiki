---
title: "Prefer small crates"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Structuring a Project"]
related_syntax: []
see_also: ["Crates", "Workspaces", "Dependency management & semver", "Compose structs"]
---

## Explanation

"Prefer small crates" is [crate](../modules-crates-visibility/crates.md)-level
advice, not a language feature: given a choice, split a growing codebase
into several small, focused crates rather than letting one crate
accumulate unrelated responsibilities — an HTTP client, a database
layer, CLI parsing, and core business logic all living in one `lib.rs`.
It's the crate-boundary analogue of keeping a single function or module
focused on one job, applied one level higher up the project's structure.

The concrete payoffs follow directly from what a crate boundary actually
is in Rust: the smallest unit `rustc` compiles as a single job, and the
smallest unit Cargo versions independently. A small, focused crate
recompiles quickly on its own and doesn't force a rebuild of unrelated
code when it changes; a downstream user who needs only the HTTP client
doesn't have to pull in — and compile — the database layer too; and each
crate gets its own [semver](../modules-crates-visibility/dependency-management-and-semver.md)
number, so a breaking change to the database layer doesn't force a
version bump on an unrelated CLI-parsing crate that happens to live in
the same repository. None of this requires publishing anything
externally — a [workspace](../modules-crates-visibility/workspaces.md)
keeps several small crates developed and versioned together in one
repository via `path` dependencies.

The judgment call is where to draw the line: splitting too aggressively
has its own tax — more `Cargo.toml` files to maintain, more inter-crate
`pub` surfaces to keep stable, and a larger dependency graph for
consumers who now need three tiny crates instead of one. The idiom is a
default lean, not an absolute rule; a single crate at the very start of
a project is entirely reasonable, and the split usually earns its cost
once a genuinely separable concern — a parser, a reusable core library,
a plugin API — becomes visible on its own.

## Basic usage example

This example shows a crate's manifest alongside its entire `lib.rs`, to
make the size of its public surface visible at a glance.

```
# shop-pricing/Cargo.toml — a small, focused crate: only pricing logic, no HTTP/DB/CLI code
[package]
name = "shop-pricing"
version = "0.1.0"

[dependencies]

# shop-pricing/src/lib.rs
pub fn discount_price(price_cents: u32, percent_off: u8) -> u32 { // <- the crate's entire public surface: one focused concern
    price_cents - (price_cents * percent_off as u32 / 100)
}
```

## Best practices & deeper information

### Scenario: Designing a public API

An e-commerce backend's single crate has grown to mix HTTP handling,
database access, and pricing logic; splitting the pricing logic into its
own crate the moment a second binary — a nightly batch repricing job —
needs it avoids either duplicating the function or pulling in the whole
web server as a dependency.

```
# shop-api/Cargo.toml — the web server, now just orchestration
[dependencies]
shop-pricing = { path = "../shop-pricing" } # <- depends on the small, focused crate instead of duplicating its logic

# shop-repricing-job/Cargo.toml — a separate nightly batch binary
[dependencies]
shop-pricing = { path = "../shop-pricing" } # <- same small crate, reused without pulling in the whole web server

# shop-pricing/src/lib.rs
pub fn discount_price(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}
```

**Why this way:** keeping the pricing logic in its own crate means
neither consumer depends on code it doesn't use, and either can evolve
independently — exactly the case the
[Rust Design Patterns' "Prefer small crates" idiom](https://rust-unofficial.github.io/patterns/idioms/prefer-small-crates.html)
argues for; a [workspace](../modules-crates-visibility/workspaces.md)
is what makes a split like this practical without publishing anything.

## Embedded Rust Notes

**Full support.** The idiom is a build-system and project-structuring
concern with no runtime footprint, so it applies just as much to
`#![no_std]` firmware as to hosted applications — arguably more so:
smaller crates make it far easier to audit and disable a dependency's
`std`-requiring features individually (`default-features = false` per
crate) than untangling one large crate that mixes `std`-dependent and
`no_std`-safe code paths together.

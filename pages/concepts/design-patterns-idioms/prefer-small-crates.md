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

## Explanation (Embedded)

The principle carries over unchanged — smaller, focused crates compile
faster and version independently regardless of target — and the
embedded-specific angle on top of that is modest but real: a small,
single-purpose crate is far easier to audit for `#![no_std]`
compatibility and code-size impact than a large one. A crate that only
does one thing has a small, reviewable list of dependencies and feature
flags to check for accidental `std` requirements (a transitive
dependency pulling in `std::time` or a heap-backed collection can quietly
make an otherwise-`no_std`-safe crate unusable in firmware); the same
audit on a large, multi-purpose crate means tracing which of its many
features actually need `std` and which don't, often without any
per-feature `no_std` documentation to go on. Splitting also makes the
flash-budget cost of a dependency visible per concern instead of
bundled: pulling in a small `crc` crate for checksum logic has an
obviously boundable code-size cost, where pulling in one large
"utilities" crate for the same feature risks dragging in unrelated code
paths the linker may or may not manage to dead-strip.

This is genuinely the same argument as the classic page, just applied
with an extra reason to care: on a hosted target, an oversized dependency
mostly costs disk space and compile time; on a flash-constrained target,
it can be the difference between an image that fits and one that
doesn't.

## Basic usage example (Embedded)

```
# sensor-crc/Cargo.toml — a small, no_std-compatible crate: one focused concern, easy to audit
[package]
name = "sensor-crc"
version = "0.1.0"

[dependencies]

# sensor-crc/src/lib.rs
#![no_std]

pub fn checksum(data: &[u8]) -> u8 { // <- the crate's entire public surface: no std, no allocator, easy to verify
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A sensor driver crate needs to stay `#![no_std]` so it can run on a
microcontroller, but the application binary that logs its readings to a
file during development needs `std`; splitting the two into separate
crates keeps the driver's `no_std` guarantee auditable on its own,
instead of buried inside one crate that mixes both.

```
# sensor-driver/Cargo.toml — small and no_std-only; nothing here can accidentally pull in std
[package]
name = "sensor-driver"

[dependencies]

# sensor-driver/src/lib.rs
#![no_std]

pub struct Reading {
    pub celsius: f32,
}

pub fn read() -> Reading { // <- the driver's entire public surface, verifiably no_std
    Reading { celsius: 21.5 }
}

# host-logger/Cargo.toml — a separate std-only crate for development-time logging
[dependencies]
sensor-driver = { path = "../sensor-driver" } # <- depends on the small no_std crate; doesn't force std on it
```

**Why this way:** keeping `sensor-driver` in its own crate makes "is this
still `#![no_std]`-safe" a one-crate question a reviewer (or CI running
`cargo build --target thumbv7em-none-eabihf`) can answer in isolation,
where a single crate mixing driver logic and `std`-based logging would
make that check depend on which feature flags happen to be enabled — the
same
[Rust Design Patterns' "Prefer small crates" idiom](https://rust-unofficial.github.io/patterns/idioms/prefer-small-crates.html)
argument, with `no_std` audit scope as the embedded-specific payoff on
top of faster rebuilds and independent versioning.

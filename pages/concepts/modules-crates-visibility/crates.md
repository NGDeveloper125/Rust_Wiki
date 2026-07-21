---
title: "Crates"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project"]
related_syntax: [crate, extern, use, pub]
see_also: ["Modules", "Workspaces", "Cargo & Cargo.toml"]
---

## Explanation

A crate is the smallest unit the Rust compiler treats as a single
compilation — one call to `rustc` in, one library or executable out — and
also the unit Cargo publishes to crates.io. A crate is either a *binary
crate*, rooted at `src/main.rs`, which produces a runnable executable, or
a *library crate*, rooted at `src/lib.rs`, which produces something other
crates can depend on. Where [modules](modules.md) organize the *inside*
of a crate, the crate itself is the boundary the compiler and the
package ecosystem both actually operate on.

Every crate has exactly one root module — the top of `main.rs` or
`lib.rs` — and that root is itself addressable with the `crate` keyword:
`crate::some_function` always names a path starting from this crate's
root, no matter how deeply nested the code writing it is. That gives a
crate an unambiguous way to refer to "my own top level" that doesn't
depend on where in the module tree the reference happens to be written.

The crate boundary is also the outer limit of one particular kind of
privacy: an item marked `pub(crate)` is visible everywhere inside the
crate but nowhere outside it, which is why [visibility &
privacy](visibility-and-privacy.md) and the crate concept are so tightly
paired — a crate is the largest scope something can be shared across
while still being completely absent from that crate's public API.

Crates are also the unit dependency management operates on: each entry
under `[dependencies]` in a crate's [Cargo.toml](cargo-and-cargo-toml.md)
names another crate, Cargo compiles it once as its own unit and links it
in, and a single crate carries exactly one name, one
[semver version](dependency-management-and-semver.md), and one published
identity on crates.io. A large project made of several crates that are
still developed together, rather than published as independent
dependencies, is what a [workspace](workspaces.md) is for.

## Basic usage example

```
// src/lib.rs — this file is the crate root
pub fn greet(name: &str) -> String {   // <- part of this crate's public API
    format!("Hello, {name}!")
}

pub mod inventory {
    pub fn describe() -> String {
        crate::greet("inventory")      // <- `crate::` names this crate's root, from anywhere inside it
    }
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A library crate's `lib.rs` is the one file whose top-level items define
the crate's entire public surface — everything a caller can reach starts
there, however the implementation is organized underneath.

```
// src/lib.rs
mod client;                       // <- private: an implementation detail
mod config;                       // <- private: an implementation detail

pub use client::WeatherClient;    // <- the crate's public API is curated here, not scattered
pub use config::Config;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

**Why this way:** keeping the internal modules private and re-exporting a
short, deliberate list from the crate root means the crate's actual
public surface can be read at a glance from one file, matching the
curation the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends for a stable public API.

### Scenario: Testing

Integration tests in a crate's `tests/` directory are each compiled as
their own separate crate that depends on the library crate the same way
an external user would — so they can only exercise what's actually
`pub`.

```
// src/lib.rs
pub fn discount_price(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}

fn round_to_nearest_cent(value: f64) -> u32 { // <- private: invisible outside this crate
    value.round() as u32
}

// tests/pricing.rs — compiled as its own crate, linked against weather_api's public API
use weather_api::discount_price;              // <- only `pub` items are reachable here

#[test]
fn applies_discount() {
    assert_eq!(discount_price(2000, 10), 1800);
}
```

**Why this way:** because each file under `tests/` links against the
library crate exactly like any outside consumer, integration tests double
as a check that the crate's public API is actually usable on its own,
per the
[Rust Book's chapter on test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).

## Embedded Rust Notes

**Full support.** A crate is a compile-time and link-time unit, not a
runtime one, so it works identically when targeting bare metal. A
`#![no_std]` firmware crate is an ordinary crate from Cargo's point of
view — it just declares `#![no_std]` at its crate root and typically
depends on other crates with `default-features = false` so they don't
pull in `std`-dependent code paths.

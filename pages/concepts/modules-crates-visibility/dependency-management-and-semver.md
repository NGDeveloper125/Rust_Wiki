---
title: "Dependency management & semver"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project"]
related_syntax: []
see_also: ["Cargo & Cargo.toml", "Crates"]
---

## Explanation

Dependency management is how a crate declares what other crates it needs
— in [Cargo.toml's](cargo-and-cargo-toml.md) `[dependencies]` table — and
how Cargo turns those declarations into one concrete, locked set of
versions recorded in `Cargo.lock`. Semantic versioning (semver) is the
convention that gives the version numbers meaning: a version string like
`1.4.2` is `MAJOR.MINOR.PATCH`, and Cargo's default requirement syntax
(writing `"1.4"` really means `^1.4`) allows any later `1.x` release
automatically, on the promise that a crate won't break existing callers
within the same major version.

Not every change is equally safe under that promise. Adding a new public
function, adding a variant to a `#[non_exhaustive]` enum, or adding a
default-implemented trait method are typically minor-version-safe
additions. Removing or renaming a public item, changing a public
function's parameters or return type, or adding a required field to a
public struct are breaking changes that call for a major-version bump.
This is where semver connects directly back to
[visibility & privacy](visibility-and-privacy.md): only `pub` items are
part of the semver contract at all — private and `pub(crate)` items can
change freely between patch releases, since no external
[crate](crates.md) can be depending on them.

`Cargo.lock` plays a different role depending on what kind of crate owns
it: an application (binary) crate commits its lockfile so every build —
every developer's machine, every CI run — resolves to the exact same
dependency versions. A library crate still generates a lockfile locally
but conventionally doesn't commit it, because it's the consuming
application's lockfile that actually matters at build time; the
library's real job is to state correct, honest semver *ranges*, not to
pin exact versions on behalf of whoever eventually depends on it.

Version requirement syntax has a few forms worth knowing at a glance:
caret requirements (the default — `"1.2"` means `^1.2`, any `1.x >= 1.2`),
tilde requirements (`"~1.2"`, patch-only updates within `1.2.x`), and
exact requirements (`"=1.2.3"`, one specific version and nothing else).
The whole system only works because crates.io refuses to let a published
version's contents ever be overwritten — without that guarantee, a
version number wouldn't reliably mean anything.

## Basic usage example

```
[dependencies]
serde = "1.0"          # <- caret requirement: any 1.x >= 1.0, Cargo picks the newest compatible
regex = "~1.10"        # <- tilde requirement: 1.10.x patch updates only, not 1.11
once_cell = "=1.19.0"  # <- exact requirement: pins one specific version
```

## Best practices & deeper information

### Scenario: Designing a public API

Adding a required parameter to an existing public function is a breaking
change under semver even though it can look like a small edit, because
every caller's existing code stops compiling against the new signature.

```
// v1.2.0 — published
pub fn fetch_forecast(city: &str) -> Forecast {
    // ...
}

// v1.3.0 — AVOID: reshapes an existing signature, breaks every caller
pub fn fetch_forecast(city: &str, units: Units) -> Forecast {
    // ...
}

// v1.3.0 — PREFER: additive; the old signature keeps compiling unchanged
pub fn fetch_forecast(city: &str) -> Forecast {
    fetch_forecast_with_units(city, Units::Metric)
}

pub fn fetch_forecast_with_units(city: &str, units: Units) -> Forecast { // <- new function, old one untouched
    // ...
}
```

**Why this way:** a version number is a promise to callers, not just a
label — reshaping an existing public signature breaks that promise even
when the change feels minor, which is why
[Effective Rust](https://effective-rust.com/) treats additive API
evolution, rather than editing existing signatures in place, as the
default way to grow a stable crate's public surface.

## Explanation (Embedded)

Semver and `Cargo.lock` work identically on an embedded target; the
genuinely embedded-specific concern is that `#![no_std]` compatibility
becomes a hard dependency-selection filter, not just a preference. A
crate that transitively pulls in `std` — even through one dependency
three levels down — simply fails to compile for a bare-metal target like
`thumbv7em-none-eabihf`, since there's no OS underneath to provide it.
This makes checking whether a candidate crate (and everything it depends
on) supports `#![no_std]`, usually via an opt-out `std` Cargo feature,
part of picking a dependency at all, not just of tuning it afterward.

## Basic usage example (Embedded)

```
[dependencies]
heapless = { version = "0.8", default-features = false }  # <- no_std-compatible fixed-capacity collections
serde = { version = "1", default-features = false }        # <- opts out of serde's default `std` feature
```

## Best practices & deeper information (Embedded)

### Scenario: Selecting a dependency for a `#![no_std]` crate

A firmware crate needs a JSON-like serialization story, and `serde` is
the obvious candidate — but pulling it in with its defaults would
silently drag in `std` and break the bare-metal build.

```
// AVOID: default features enabled — pulls in `std`, won't compile for thumbv7em-none-eabihf
serde = "1"

// PREFER: opts out of the default `std` feature; serde derive still works under `#![no_std]`
serde = { version = "1", default-features = false }
```

**Why this way:** `serde`'s `std` feature is on by default because most
of its users are hosted; disabling it and checking a candidate crate's
docs for `no_std` support before adding the dependency avoids
discovering a hard compile failure only after the code depending on it
is already written, which the
[Embedded Rust Book](https://docs.rust-embedded.org/book/) calls out as
a routine first check for any candidate crate.

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

## Embedded Rust Notes

**Full support.** Dependency resolution and semver work identically
regardless of target — Cargo doesn't distinguish "embedded" dependencies
from any other kind. The idiom worth knowing is that many `no_std`-
compatible crates expose a `std` Cargo feature enabled by default, so the
same published crate works both hosted and on bare metal; an embedded
consumer opts out with `default-features = false` rather than depending
on a separate "embedded edition" of the crate.

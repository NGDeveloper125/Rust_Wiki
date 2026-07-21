---
title: "Anti-pattern: #[deny(warnings)]"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Anti-patterns", "Design Patterns & Idioms", "Testing & Documenting Code"]
related_syntax: []
see_also: ["Custom error types"]
---

## Explanation

`#![deny(warnings)]` at a crate root turns every lint warning the
compiler (and, if run together, Clippy) currently knows how to emit into
a hard compile error. It reads like a strong statement of code quality —
"this crate has zero warnings, and it will stay that way" — and it costs
one line to add, which is exactly what makes it appealing.

The trouble is what "every warning" actually includes: not just the
lints that exist today, but every lint a *future* compiler or Clippy
release adds. Rust regularly introduces new warn-by-default lints across
releases as the language and its static analysis improve. A crate with
`#![deny(warnings)]` that compiled cleanly last month can fail to build
outright the moment a routine `rustup update` (or a CI runner picking up
a newer toolchain) ships a new lint that happens to fire on code that
hasn't changed at all. For a library, that failure doesn't just affect
the author — it breaks the build for every downstream crate that
compiles it from source on a newer toolchain than the one the author
tested against, which is a far worse outcome than the warning it was
trying to prevent.

It also removes any ability to make a deliberate, temporary exception.
Ordinary warnings can be addressed on the maintainer's own schedule, or
suppressed for one specific, justified line with `#[allow(...)]`; a
blanket `deny` at the crate root offers no such granularity — any new
warning anywhere in the crate is an immediate hard failure blocking every
build until it's fixed, whether or not the maintainer had any advance
notice it was coming.

The idiomatic alternative keeps the intent — no warnings slipping through
unnoticed — without the toolchain-version fragility: deny an explicit,
curated list of specific lints the project actually cares about
(`#![deny(unused_must_use)]`, `#![deny(unsafe_op_in_unsafe_fn)]`, and so
on) rather than the open-ended `warnings` group, so a new lint appearing
in a future compiler is something the crate opts into deliberately rather
than something that silently starts breaking its build. Where the real
goal is "don't let warnings merge to main," that belongs in CI as an
explicit, separate gate — `cargo build` (or `cargo clippy`) invoked with
`RUSTFLAGS="-D warnings"` or `-- -D warnings` for that one job — rather
than baked permanently into the source every downstream consumer
compiles with.

## Basic usage example

```
// A curated, explicit deny list instead of the open-ended `warnings` group:
#![deny(unused_must_use)] // <- PREFER: names exactly the lint(s) this crate wants to be a hard error

fn parse_count(input: &str) -> Result<u32, std::num::ParseIntError> {
    input.trim().parse()
}

fn main() {
    let _ = parse_count("42"); // handling the Result explicitly satisfies unused_must_use
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A library wants a genuinely warning-free build enforced, without that
guarantee turning into a broken build for every downstream user the day
a new compiler release adds a lint the crate has never seen before.

```
// AVOID: any future lint the compiler or Clippy adds becomes an immediate hard error for every consumer
// #![deny(warnings)]

// PREFER: an explicit, curated list — new lints from a future toolchain don't silently break the build
#![deny(unused_must_use, unused_imports)]
#![warn(missing_docs)] // <- a lint the project tracks but isn't ready to make a hard error yet

/// Parses a port number from a configuration string.
pub fn parse_port(input: &str) -> Result<u16, std::num::ParseIntError> {
    input.trim().parse()
}
```

**Why this way:** `#![deny(warnings)]` denies the entire, ever-growing
`warnings` lint group rather than a fixed set, so a compiler or Clippy
upgrade that adds one new warn-by-default lint can turn unrelated,
previously-clean code into a hard build failure with no warning; the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/deny-warnings.html)
documents this exact fragility and recommends denying specific, named
lints instead — the same reasoning behind Clippy's own guidance to scope
[`#[allow]`/`#[deny]`](https://doc.rust-lang.org/rustc/lints/levels.html)
attributes to precise lint names rather than whole groups when the goal
is a stable, predictable build.

## Embedded Rust Notes

**Full support.** This is purely a build-time lint-configuration concern
with no runtime behavior at all, so the guidance is identical on
`#![no_std]` targets — if anything it matters more there, since embedded
crates are often pinned to a specific toolchain for hardware-support
reasons, and a stray `#![deny(warnings)]` combined with a toolchain bump
is an easy way to break a build no one intended to touch.

---
title: "Clippy & rustfmt"
area: "Testing & Tooling"
embedded_support: full
groups: ["Testing & Tooling", "Testing & Documenting Code"]
related_syntax: ["#[allow(...)]", "#[warn(...)]", "#[rustfmt::skip]"]
see_also: ["Doc tests", "Derivable traits"]
---

## Explanation

Clippy and rustfmt are Rust's two cargo-integrated code-quality tools,
and they answer two different questions. Rustfmt (`cargo fmt`) answers
"how is this laid out" — it rewrites source code into one canonical
formatting style (indentation, line breaks, spacing) so that code review
and diffs are about substance rather than whitespace, and so nobody has
to debate formatting style by hand. Clippy (`cargo clippy`) answers "is
this idiomatic, or possibly wrong" — it's a linter with several hundred
additional lints beyond what `rustc` checks on its own, catching things
the compiler considers perfectly valid but that are usually a mistake, an
unnecessary allocation, or a non-idiomatic way to express something
simpler.

Clippy's lints are grouped by confidence and intent: `correctness` lints
flag code that is likely an outright bug, `style` and `complexity` lints
suggest simpler equivalent code, `perf` lints flag likely performance
issues, and the `pedantic`/`nursery`/`restriction` groups hold stricter or
more opinionated lints that are opt-in rather than on by default. Each
lint has a stable name (`clippy::needless_clone`,
`clippy::unwrap_used`, …) that can be silenced per-item with
`#[allow(...)]` or escalated project-wide with `#![warn(...)]` or
`#![deny(...)]` at the crate root — the same attribute mechanism used to
control `rustc`'s own built-in lints, just under the `clippy::` tool-lint
namespace.

Both tools are meant to run continuously rather than occasionally: most
teams wire `cargo clippy -- -D warnings` and `cargo fmt --check` into CI
alongside `cargo test`, so a lint regression or a formatting drift fails
the build the same way a broken test would, instead of being caught (or
missed) in code review by eye.

## Basic usage example

```
#![warn(clippy::all)] // <- enables clippy's default lint group for this crate

fn double(x: i32) -> i32 {
    x * 2
}
```

## Best practices & deeper information

### Scenario: Designing a public API

Clippy's `unwrap_used` lint flags a public function that panics on bad
input via `.unwrap()`, pushing the design toward a `Result`-returning
signature that lets the caller decide how to handle failure instead.

```
// AVOID: panics inside a public function whenever parsing fails
pub fn parse_config_avoid(raw: &str) -> u32 {
    raw.parse().unwrap() // <- clippy::unwrap_used flags this in library code
}

// PREFER: the caller gets to decide what a bad value means
pub fn parse_config(raw: &str) -> Result<u32, std::num::ParseIntError> {
    raw.parse() // <- no unwrap: failure is part of the signature, not a panic
}
```

**Why this way:** a panic buried inside a library function takes the
error-handling decision away from the caller entirely; the
[`clippy::unwrap_used`](https://rust-lang.github.io/rust-clippy/master/#unwrap_used)
lint (part of the `restriction` group, commonly enabled for library
crates) exists specifically to surface this before it ships.

### Scenario: Documenting an API

`#![warn(missing_docs)]` (a built-in `rustc` lint, commonly enabled
alongside clippy's own lints) turns "this public item has no `///`
comment" into a build warning, catching undocumented API surface before
it reaches a published crate.

```
#![warn(missing_docs)] // <- every public item below now needs a /// comment

/// The total price of an order, in whole cents.
pub struct OrderTotal(pub u32); // <- documented: satisfies the lint

pub struct ShippingAddress { // <- missing_docs warns: no /// comment on this public item
    pub line1: String,
}
```

**Why this way:** letting undocumented public items compile silently is
how documentation gaps accumulate unnoticed in a growing crate; the
[Rust API Guidelines' C-CRATE-DOC](https://rust-lang.github.io/api-guidelines/documentation.html#crate-level-docs-are-thorough-and-include-examples-c-crate-doc)
expectation is easiest to hold to when the compiler itself enforces it
rather than relying on reviewers to notice a missing comment.

### Scenario: Numeric computation

`clippy::float_cmp` flags direct `==` comparison between floating-point
values — a comparison that looks correct but is fragile because
arithmetic on floats rarely produces exactly the bit pattern a reader
expects.

```
fn readings_match_avoid(a: f64, b: f64) -> bool {
    a == b // AVOID: clippy::float_cmp — exact equality on floats is rarely what's intended
}

fn readings_match(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() <= tolerance // PREFER: compare within a tolerance instead
}
```

**Why this way:** floating-point rounding means two values that are
"the same" mathematically can differ in their last bit after arithmetic,
so exact `==` silently produces false negatives; the
[`clippy::float_cmp`](https://rust-lang.github.io/rust-clippy/master/#float_cmp)
lint exists to catch this before it becomes an intermittent bug in
sensor or scientific code.

## Embedded Rust Notes

**Full support.** Clippy and rustfmt operate on source code through
Cargo, entirely independent of the compilation target — they run exactly
the same way, with the same lints and formatting rules, whether the crate
targets a hosted OS or `#![no_std]` bare metal. Neither tool needs `std`,
an allocator, or any runtime support to do its job, since both work at
the level of source text and the compiler's own parsed representation of
it, not the compiled artifact.

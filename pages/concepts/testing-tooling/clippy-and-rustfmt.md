---
title: "Clippy & rustfmt"
area: "Testing & Tooling"
embedded_support: full
groups: ["Testing & Tooling", "Testing & Documenting Code"]
related_syntax: ["#[allow(...)] / #[warn(...)] / #[deny(...)] / #[forbid(...)]"]
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

## Explanation (Embedded)

Clippy and rustfmt run identically regardless of target. Both operate on
source code — rustfmt reformats source text directly, and clippy lints
against the compiler's parsed representation of it (roughly, the same
stage `rustc` itself analyzes) — rather than on anything produced by
codegen or linking. Cross-compiling to `thumbv7em-none-eabihf` instead of
the host triple doesn't change what either tool sees: `cargo clippy
--target thumbv7em-none-eabihf` and `cargo fmt` run the same lints and
apply the same formatting rules they would on any hosted crate, with no
embedded-specific behavior in either tool.

What does shift is which of clippy's existing, general-purpose lints
carry more weight in firmware. Nothing about `#![no_std]` adds
embedded-specific lints, but restriction-group lints like
`clippy::unwrap_used`, `clippy::indexing_slicing`, and
`clippy::arithmetic_side_effects` — all of which flag places a panic
could occur — are worth enabling more readily on a target where a panic
often means an abort-and-reset with no attached terminal to report it,
rather than a caught unwind a hosted process can log and recover from.
That's a difference in which lints a project chooses to turn on, not a
difference in what the tools themselves do.

## Basic usage example (Embedded)

```
#![no_std]
#![warn(clippy::arithmetic_side_effects)] // <- an ordinary clippy lint; runs the same under any --target

pub fn scale_reading(raw: u16, factor: u16) -> u16 {
    raw.saturating_mul(factor) // <- avoids the panic-on-overflow arithmetic the lint flags
}
```

## Best practices & deeper information (Embedded)

### Scenario: Numeric computation

A calibration calculation on sensor data uses plain arithmetic that could
overflow; `clippy::arithmetic_side_effects` flags it the same way it
would in any hosted crate, but the consequence of leaving it unfixed is
more severe on a target where an overflow panic resets the device rather
than getting caught by a supervising process.

```
#![no_std]
#![warn(clippy::arithmetic_side_effects)]

// AVOID: an overflowing add here panics, and on target that means a reset, not a caught error
fn calibrate_avoid(raw: u16, offset: u16) -> u16 {
    raw + offset // <- clippy::arithmetic_side_effects flags this
}

// PREFER: overflow is handled explicitly instead of panicking
fn calibrate(raw: u16, offset: u16) -> u16 {
    raw.saturating_add(offset)
}
```

**Why this way:** the lint itself is not embedded-specific — it fires
identically on a hosted crate — but a panic from unchecked arithmetic is
markedly more expensive on a target with no unwind-catching supervisor
to report it, which is a reason to enable this ordinarily-opt-in
restriction lint more readily in firmware; see
[`clippy::arithmetic_side_effects`](https://rust-lang.github.io/rust-clippy/master/#arithmetic_side_effects)
in the clippy lint list.

### Scenario: Handling and propagating errors

A peripheral-init routine that panics via `.unwrap()` on a hardware
failure leaves a firmware author with only a reset to recover from;
`clippy::unwrap_used` flags this exactly as it would in hosted code,
pushing the driver toward a `Result`-returning constructor instead.

```
#![no_std]
#![warn(clippy::unwrap_used)]

// AVOID: any I2C failure here panics the whole firmware
fn init_sensor_avoid(i2c: &mut impl embedded_hal::i2c::I2c) {
    i2c.write(0x76, &[0x00]).unwrap(); // <- clippy::unwrap_used flags this in library code
}

// PREFER: the caller decides how to handle an init failure
fn init_sensor(i2c: &mut impl embedded_hal::i2c::I2c) -> Result<(), embedded_hal::i2c::ErrorKind> {
    i2c.write(0x76, &[0x00]).map_err(|_| embedded_hal::i2c::ErrorKind::Other)
}
```

**Why this way:** the lint is the same one described in this page's
classic "Designing a public API" scenario above, unchanged for an
embedded target — but returning `Result` matters even more in firmware,
where a caller may want to retry a transient bus error rather than let a
panic-triggered reset discard in-progress state.

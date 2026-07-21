---
title: "Doc tests"
area: "Testing & Tooling"
embedded_support: partial
groups: ["Testing & Tooling", "Testing & Documenting Code", "Testing"]
related_syntax: ["///", "//!"]
see_also: ["Unit tests", "Integration tests"]
---

## Explanation

A doc test is a fenced code block written inside a documentation comment
— [`///`](../../syntax/comments/outer-line-doc-comment.md) above an item,
or `//!` at the top of a module or crate — that `cargo test` compiles and
runs as an executable example, in addition to `rustdoc` rendering it as
prose. Nothing special has to be written to opt in: by default, any
fenced block inside a `///` comment on a public item of a library crate
*is* a doc test, which is what makes doc comments unusual among Rust's
documentation tools — they are simultaneously human-facing prose and a
regression check.

The mental model worth keeping is that a doc test is really testing the
*documentation*, not just the code: it exists to guarantee that the
example a reader sees on `docs.rs` still compiles and behaves exactly as
shown. If a function's behavior changes and nobody updates its doc
comment's example, `cargo test` fails — turning what would otherwise be
silently stale documentation into a build failure, which is precisely the
gap [unit tests](unit-tests.md) and [integration tests](integration-tests.md)
don't cover, since neither one looks at prose.

Each doc test is compiled as its own tiny standalone program, implicitly
wrapped in a `fn main() { ... }` unless the block already defines one, and
linked against the crate's public API exactly like an external caller —
which is why a doc test can only reference `pub` items, the same
constraint an [integration test](integration-tests.md) has. Annotations
placed right after the opening triple backtick change how a block is
treated: `no_run` compiles it but skips execution (useful for examples
with real side effects), `ignore` skips it entirely, and `should_panic`
asserts the example panics rather than returning normally.

## Basic usage example

```
/// Converts a Celsius temperature to Fahrenheit.
///
/// ```
/// assert_eq!(sensors::celsius_to_fahrenheit(0.0), 32.0); // <- this block compiles and runs under `cargo test`
/// ```
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}
```

## Best practices & deeper information

### Scenario: Documenting an API

Lines prefixed with a single `#` inside a doc test's fenced block are
compiled and run but hidden from the rendered documentation — useful for
setup a reader doesn't need to see, like importing a type the surrounding
prose already named.

```
/// Parses a sensor reading line like `"7,21.5"` into an id and a Celsius value.
///
/// ```
/// # use sensors::SensorReading;
/// let reading = sensors::parse_reading("7,21.5").unwrap();
/// assert_eq!(reading, SensorReading { id: 7, celsius: 21.5 }); // <- what the reader actually sees
/// ```
#[derive(Debug, PartialEq)]
pub struct SensorReading {
    pub id: u32,
    pub celsius: f64,
}

pub fn parse_reading(line: &str) -> Option<SensorReading> {
    let (id, celsius) = line.split_once(',')?;
    Some(SensorReading { id: id.parse().ok()?, celsius: celsius.parse().ok()? })
}
```

**Why this way:** hiding plumbing behind `#` keeps the rendered example
focused on the one line that matters to a reader, while the hidden line
still keeps the block honestly compiling — the
[rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#hiding-portions-of-the-example)
documents `#`-prefixed lines as exactly this: executed, but not displayed.

### Scenario: Handling and propagating errors

A doc test whose example needs the `?` operator requires an explicit
`fn main` with a `Result` return type — the implicit wrapper `cargo test`
generates otherwise returns `()`, which `?` can't propagate into.

```
/// Parses a duration string like `"30s"` into whole seconds.
///
/// ```
/// # fn run() -> Result<(), std::num::ParseIntError> {
/// let seconds = config::parse_duration_seconds("30s")?; // <- `?` needs the enclosing fn to return a Result
/// assert_eq!(seconds, 30);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub fn parse_duration_seconds(input: &str) -> Result<u32, std::num::ParseIntError> {
    input.trim_end_matches('s').parse()
}
```

**Why this way:** hiding the `fn run() -> Result<...>` wrapper (and the
call that unwraps it) behind `#` lets the visible example use `?` the
same way real calling code would, instead of forcing every doc example
into an artificial `.unwrap()`-only style — a pattern the
[rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#using--in-doc-tests)
describes for exactly this case.

### Scenario: Testing

`should_panic` on a doc test asserts that the shown example panics,
turning a documented panic condition into something `cargo test` actually
checks rather than a claim in prose that could quietly go stale.

```
/// Divides `total` evenly across `shares`.
///
/// # Panics
///
/// Panics if `shares` is zero.
///
/// ```should_panic
/// sensors::split_evenly(100, 0); // <- doc test asserts this call panics
/// ```
pub fn split_evenly(total: u32, shares: u32) -> u32 {
    if shares == 0 {
        panic!("cannot split into zero shares");
    }
    total / shares
}
```

**Why this way:** documenting a panic condition under a `# Panics`
heading without a test backing it is just an unverified claim; pairing it
with a `should_panic` doc test means the documentation and the actual
behavior can't drift apart silently, matching the
[API Guidelines' C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)
expectation that panic conditions are documented at all.

## Embedded Rust Notes

**Partial support.** Doc tests compile and execute on the **host**
toolchain, the same way unit and integration tests do — there is no
mechanism to run a doc test on the target microcontroller itself. This is
still useful for `#![no_std]` crates whose public API is pure logic (a
parser, a checksum calculation): the doc example compiles against the
`no_std` crate but runs as an ordinary host program. Anything whose
correctness depends on real hardware isn't a good fit for a doc test and
is usually left as plain, non-tested prose instead, or covered separately
by an on-target framework like `defmt-test`.

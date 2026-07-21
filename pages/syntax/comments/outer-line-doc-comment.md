---
title: "/// (outer line doc comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: ["Doc tests"]
related_syntax: [line-comment, inner-line-doc-comment]
see_also: [inner-line-doc-comment]
---

## Explanation

`///` documents the item immediately **following** it, and — unlike a
plain `//` comment — is not discarded by the compiler: it's collected as
a `#[doc = "..."]` attribute and rendered into generated documentation
(`cargo doc`), as in `/// Adds two numbers together.` placed directly
above the function it describes.

The content supports Markdown, and any fenced code block inside it is, by
default, compiled and run as a **doc test** by `cargo test` — making `///`
double as both documentation and an executable example in one place.
`///` attaches to the item that follows it; intervening code or another
item breaks the association (blank lines alone don't — a doc comment
desugars to an outer `#[doc = "..."]` attribute, and whitespace between
an attribute and its item is insignificant).

## Usage examples

### Documenting a function

```
/// <- this doc comment documents the function immediately below it
/// Adds two numbers together.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Restriction:** `///` documents the next item that follows it — any
unrelated code or another item in between redirects (or breaks) the
association. Blank lines alone are harmless.

### Documenting an API

A well-formed `///` comment leads with a one-line summary (used in
listings/search), then expands with usage details — the shape `cargo doc`
and IDE hover tooltips both render well.

```
/// Parses a duration string like `"5s"` or `"10m"` into whole seconds.
///
/// # Errors
///
/// Returns [`ParseError`] if `input` has no trailing unit character or
/// the numeric portion doesn't parse as an integer.
pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    // <- everything above is `///`; it documents this fn, not the body below
    todo!()
}

pub struct ParseError;
```

The summary-then-detail shape and the `# Errors`
section heading are conventions from the
[rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
and the [API Guidelines' C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure) —
callers scanning generated docs expect failure conditions called out
explicitly, not buried in prose.

### Testing

By default, a fenced code block inside a `///` comment on a library item
is compiled and executed as a **doc test** by `cargo test` (annotations
like `no_run` or `ignore` opt out) — making the documentation double as a
regression test that fails loudly if the example ever stops compiling or
returns something different.

```
/// Adds two numbers together.
///
/// ```
/// assert_eq!(my_crate::add(2, 3), 5); // <- this block runs under `cargo test`
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

A doc test is the one kind of example guaranteed to
stay accurate — if `add`'s behavior changes and the example wasn't
updated, `cargo test` fails instead of leaving stale documentation on the
page.

### Designing a public API

Cross-referencing related items with an intra-doc link (`` [`Type`] ``)
lets a reader jump straight from one API's docs to another's, without
hand-written URLs that rot as the crate evolves.

```
/// The parsed result of [`parse_duration`].
///
/// See also [`Duration`](std::time::Duration) for the standard-library
/// equivalent once parsing is done.
pub struct ParsedDuration {
    pub seconds: u64,
}
```

Intra-doc links are resolved and checked by `rustdoc`
itself — a broken reference becomes a build-time warning instead of a
silently dead link, which the
[rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html)
recommends over manual markdown links whenever the target is another
item in the same doc set.

## Embedded Rust Notes

**Full support.** `#[doc = "..."]` generation works identically in
`#![no_std]`. One practical note: doc tests still compile and run on the
**host** toolchain by default, not on the target microcontroller — fine
for documenting pure logic, but an example that needs real hardware
typically has to be written as plain (non-tested) text instead.

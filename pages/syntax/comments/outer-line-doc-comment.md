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
(`cargo doc`).

```
/// Adds two numbers together.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

The content supports Markdown, and any fenced code block inside it is, by
default, compiled and run as a **doc test** by `cargo test` — making `///`
double as both documentation and an executable example in one place.
`///` must appear directly before the item it documents; a blank line or
unrelated code between them breaks the association.

## Basic usage example

```
/// <- this doc comment documents the function immediately below it
/// Adds two numbers together.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Restriction:** `///` must sit directly above the item it documents —
no blank line or unrelated code in between — or the association is lost.

## Embedded Rust Notes

**Full support.** `#[doc = "..."]` generation works identically in
`#![no_std]`. One practical note: doc tests still compile and run on the
**host** toolchain by default, not on the target microcontroller — fine
for documenting pure logic, but an example that needs real hardware
typically has to be written as plain (non-tested) text instead.

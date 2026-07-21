---
title: "#[meta] / #![meta]"
kind: attribute
embedded_support: full
groups: ["Core Syntax", "Macros & Metaprogramming"]
related_concepts: []
related_syntax: ["///", "//!", "#[derive(...)]", "#[cfg(...)]"]
see_also: ["///", "//!"]
---

## Explanation

An attribute attaches metadata to a piece of Rust code, consumed by the
compiler, a lint pass, or a macro — never by the program's own runtime
logic. Every specific attribute covered elsewhere in this wiki
(`#[derive(...)]`, `#[cfg(...)]`, `#[test]`, `#[repr(...)]`, and dozens
more) is one particular *consumer* of this single shared syntax; this
page covers the grammar and placement rules all of them share.

**Outer vs. inner — the same split as `///` vs `//!`.** An **outer**
attribute, written `#[...]`, applies to the single item **immediately
following** it — a `fn`, `struct`, `enum`, `mod`, `impl` block, or
statement. An **inner** attribute, written `#![...]`, applies to the item
it is **inside of**: placed at the top of a function body it applies to
that function, placed at the top of a module file it applies to the
module, and placed at the very top of a crate root (`lib.rs`/`main.rs`,
before any item) it applies to the **whole crate**. This is exactly the
outer/inner contrast [`///`](../comments/outer-line-doc-comment.md) and
[`//!`](../comments/inner-line-doc-comment.md) already draw for doc
comments — and it's not a coincidence: a `///` doc comment literally
desugars into an outer `#[doc = "..."]` attribute, and `//!` into an inner
`#![doc = "..."]` one. Doc comments are attributes wearing different
clothes.

**Grammar.** An attribute's contents (inside the `#[...]`/`#![...]`
delimiters) is a **path** — most often a single identifier (`derive`,
`test`, `repr`), occasionally a longer path like `diagnostic::on_unimplemented`
— optionally followed by one of three forms:

- **Nothing at all** — a bare path, like `#[test]` or `#![no_std]`.
- **Parenthesized arguments** — `#[derive(Debug, Clone)]`,
  `#[cfg(target_os = "linux")]`, `#[repr(C, packed)]`. Everything inside
  the parentheses is itself made of **meta items**, each one either a bare
  identifier (`Debug`), a `name = value` pair where `value` is a literal
  (`target_os = "linux"`), or another nested, parenthesized meta item
  (`not(feature = "std")` inside `cfg`) — meta items nest to arbitrary
  depth, which is what lets `cfg`'s `all(...)`/`any(...)`/`not(...)`
  combinators compose.
- **`= value`** directly (no parentheses) — `#[doc = "some text"]`,
  `#![recursion_limit = "256"]`. This form takes exactly one literal value
  and nothing else.

A single item can carry any number of attributes, in any order relative
to each other and to doc comments, each stacked on its own line directly
above the item.

Attributes are how Rust handles "everything that isn't quite a keyword":
conditional compilation ([`#[cfg(...)]`](cfg-attribute.md)), automatic
trait implementations ([`#[derive(...)]`](derive.md)), lint control
([`#[allow(...)]`](allow-and-friends.md) and family), test discovery
([`#[test]`](test-attribute.md)), FFI linkage
([`#[repr(...)]`](repr.md), linkage attributes), and more — all built as
independent consumers of one uniform syntax rather than each getting its
own bespoke grammar. New capability tends to arrive as a new attribute
name rather than a new keyword, precisely because the attribute mechanism
is already general enough to carry it.

## Basic usage example

```
#![allow(dead_code)] // <- inner attribute: applies to the enclosing module/crate, not to what follows

#[derive(Debug)] // <- outer attribute: applies only to the struct immediately below
struct Reading {
    celsius: f64,
}
```

## Best practices & deeper information

### Scenario: Designing a public API

The same kind of item — here, a module — can be annotated from the
outside (an outer attribute on the `mod` item, from the parent's point of
view) or from the inside (an inner attribute at the top of the module's
own file/block, from the module's own point of view). Both are legal;
which one to reach for depends on who is making the statement.

```
// lib.rs (crate root):
#![allow(clippy::module_name_repetitions)] // <- inner: crate root annotating itself, before any item

#[cfg(feature = "metrics")] // <- outer: parent module gating the `metrics` module from outside
mod metrics {
    #![allow(dead_code)] // <- inner: metrics module annotating itself, from inside
    pub fn record_latency(_ms: u64) {}
}
```

**Why this way:** an outer attribute reads naturally when a *caller* of an
item is making a decision about it (whether to compile `mod metrics` at
all); an inner attribute reads naturally when an item is making a
statement about *itself* (this crate root allows a specific Clippy lint
everywhere within it) — the
[Rust Reference](https://doc.rust-lang.org/reference/attributes.html)
documents both forms as applying to "the item enclosing it" (inner) vs.
"the item following it" (outer), and idiomatic code picks whichever
direction matches who owns the decision.

### Scenario: Testing

`#[cfg(test)]` (outer, gating a whole module from the outside) and
`#[test]` (outer, marking one function inside it) are two independent
attributes cooperating on the same file — recognizing that they're both
just outer attributes with different targets clarifies why one gates a
`mod` and the other marks a `fn` rather than either one doing both jobs.

```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)] // <- outer attribute on the `mod` item: this module compiles only under `cargo test`
mod tests {
    use super::*;

    #[test] // <- outer attribute on the `fn` item: marks one function as a test case
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }
}
```

**Why this way:** keeping compilation-gating (`cfg`) and test-discovery
(`test`) as two separate, composable attributes — rather than one
attribute trying to do both — is what lets the same `#[cfg(test)]` module
also hold non-test helper functions, or lets `#[test]` be combined with
`#[should_panic]` or `#[ignore]` on the same function without a combined
grammar having to anticipate every combination in advance; see
[Unit tests](../../concepts/testing-tooling/unit-tests.md) for the full
picture of how `cargo test` uses this pairing.

## Embedded Rust Notes

**Full support.** Attribute syntax itself is core-language grammar,
processed entirely at compile time with zero runtime footprint — it works
identically whether or not the crate links `std`. `#![no_std]` is itself
just an ordinary inner attribute applied to a crate root; see
[`#![no_std]`](no-std-attribute.md) for what that specific attribute does.

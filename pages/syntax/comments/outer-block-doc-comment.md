---
title: "/** */ (outer block doc comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: ["Doc tests"]
related_syntax: [block-comment, outer-line-doc-comment, inner-block-doc-comment]
see_also: [outer-line-doc-comment]
---

## Explanation

`/** ... */` is the block-comment equivalent of `///` — it documents the
item immediately following it and participates in generated documentation
and doc tests the same way.

```
/** Adds two numbers together. */
fn add(a: i32, b: i32) -> i32 { a + b }
```

In practice `///` is far more idiomatic in the Rust ecosystem and is what
`rustfmt`/community style favors; `/** */` is rarely seen in real
codebases even though it's fully supported.

## Basic usage example

```
/** <- this block doc comment documents the function immediately below it */
fn add(a: i32, b: i32) -> i32 { a + b }
```

**Restriction:** same placement rule as `///` — it must sit directly
before the item it documents. In practice, prefer `///`
(see [`///`](outer-line-doc-comment.md)); this form exists mostly for
completeness.

## Embedded Rust Notes

**Full support.** Same as [`///`](outer-line-doc-comment.md) — no `std`
dependency, same host-vs-target doc test caveat.

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
and doc tests the same way, as in
`/** Adds two numbers together. */` placed directly above a function.

In practice `///` is far more idiomatic in the Rust ecosystem and is what
`rustfmt`/community style favors; `/** */` is rarely seen in real
codebases even though it's fully supported.

## Usage examples

### Documenting a function

```
/** <- this block doc comment documents the function immediately below it */
fn add(a: i32, b: i32) -> i32 { a + b }
```

**Restriction:** same placement rule as `///` — it must sit directly
before the item it documents. In practice, prefer `///`
(see [`///`](outer-line-doc-comment.md)); this form exists mostly for
completeness.

### Documenting an API

`/** */` documents an item exactly like `///` — the choice between them
is purely stylistic, and idiomatic Rust code overwhelmingly picks `///`.

```
/** Parses a duration string like `"5s"` into whole seconds. */
pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    // <- `/** */` above documents this fn; behaves identically to `///`
    todo!()
}

pub struct ParseError;
```

`rustfmt` and community convention treat `///` as the
default for item docs — see [`///`](outer-line-doc-comment.md) for the
full treatment (summary/detail shape, doc tests, intra-doc links), all of
which apply here unchanged. Reach for `/** */` only to match an existing
codebase's established style, not for new code.

## Embedded Rust Notes

**Full support.** Same as [`///`](outer-line-doc-comment.md) — no `std`
dependency, same host-vs-target doc test caveat.

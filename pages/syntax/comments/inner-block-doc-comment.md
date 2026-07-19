---
title: "/*! */ (inner block doc comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [block-comment, inner-line-doc-comment]
see_also: [inner-line-doc-comment]
---

## Explanation

`/*! ... */` is the block-comment equivalent of `//!` — it documents the
enclosing module/crate rather than a following item.

```
/*! This module implements the parser's tokenizer. */
```

As with `/** */` vs `///`, this form is rarely used in idiomatic Rust —
`//!` is the conventional choice — but both are equivalent to the
compiler.

## Embedded Rust Notes

**Full support.** Same as [`//!`](inner-line-doc-comment.md) — no `std`
dependency.

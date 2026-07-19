---
title: "// (line comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [block-comment, outer-line-doc-comment]
see_also: [block-comment]
---

## Explanation

`//` begins a comment that runs to the end of the line. It has zero
effect on compilation — the compiler discards it entirely before parsing
(unlike doc comments, which are collected into documentation).

```
// this line is ignored entirely
let x = 5; // so is everything after this // on this line
```

Nesting doesn't apply, since a line comment simply consumes the rest of
the line regardless of what characters follow.

## Basic usage example

```
fn main() {
    // <- this is a line comment: everything from `//` to the end of the line
    let x = 5; // comments can also trail code on the same line
    println!("{x}");
}
```

## Embedded Rust Notes

**Full support.** Pure lexical construct, discarded before compilation —
no `std` dependency.

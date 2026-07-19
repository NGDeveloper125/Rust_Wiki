---
title: "Raw string literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["String vs &str"]
related_syntax: [string-literal, escape-sequences]
see_also: [string-literal]
---

## Explanation

A raw string literal disables escape processing entirely — every
character between the quotes is taken literally, including backslashes:

```
let path = r"C:\Users\name";
let regex = r"\d+\.\d+";
```

When the string itself needs to contain a `"`, wrap it in matching `#`
delimiters — `r#"..."#` — and use as many `#` as needed to avoid ambiguity
with any `#` sequences inside the content itself (`r##"contains "# inside"##`).
Like a normal string literal, the result type is `&str`; the only
difference is how the literal's *source text* is interpreted, not the
resulting type.

## Basic usage example

```
let path = r"C:\temp\file"; // <- `r"..."`: raw string, backslashes are literal, not escapes
```

**Restriction:** `#` delimiters must be balanced and matched —
`r#"..."#` needs the same number of `#` on both sides, chosen high
enough to avoid ambiguity with any `#` sequences in the content.

## Embedded Rust Notes

**Full support.** Same as an ordinary string literal — no allocator
needed, no `std` dependency.

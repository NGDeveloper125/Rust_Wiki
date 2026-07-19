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

## Best practices & deeper information

### Scenario: Testing

A `//` comment above a non-obvious test case records *why* that specific
input is being checked, so a future reader (including yourself in six
months) doesn't have to reverse-engineer the intent from the assertion
alone.

```
#[test]
fn rejects_empty_username() {
    // regression test for #142: an empty string used to pass validation
    // <- this comment survives in source but never reaches compiled output
    let result = validate_username("");
    assert!(result.is_err());
}
```

**Why this way:** the comment explains *why* the test exists, not *what*
the code does — the assertion already says what; only the ticket/context
behind an easy-to-delete-looking test case is worth spelling out.

### Scenario: Documenting an API

`//` and `///` look almost identical but serve opposite audiences: `//`
is for the next person reading the source, `///` is for the next person
*calling* the function who may never open the source at all.

```
/// Parses a duration string like "5s" or "10m" into seconds.
pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    // AVOID: burying caller-relevant info in a // comment nobody sees
    // the trailing unit character determines the multiplier
    let (digits, unit) = input.split_at(input.len() - 1);
    // ...
}
```

**Why this way:** anything the caller needs to know (accepted formats,
error conditions, examples) belongs in a `///` doc comment — see
[`///`](outer-line-doc-comment.md) — where `cargo doc` and IDE tooltips
surface it; `//` is reserved for notes aimed at maintainers, per the
[rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html).

## Embedded Rust Notes

**Full support.** Pure lexical construct, discarded before compilation —
no `std` dependency.

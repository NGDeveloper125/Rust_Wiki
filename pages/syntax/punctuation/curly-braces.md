---
title: "{ }"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language, Structs]
related_syntax: [";"]
see_also: [";"]
---

## Explanation

`{ }` delimits a **block expression** — a sequence of statements followed
by an optional final expression:

```
let y = {
    let x = 1;
    x + 1
};
```

A block is itself an expression: it evaluates to its final expression (if
it has no trailing `;`), or to `()` otherwise. Function bodies, `if`/`else`
arms, loop bodies, and match arms are all block expressions under the
hood, which is exactly why `if`/`match` can produce values.

`{ }` is reused for a completely different purpose in `Type { field: value, ... }`
— a **struct literal** — where it delimits field initializers rather than
statements. The two uses are distinguished purely by what precedes the
brace (a type path vs. nothing), which is also why `if SomeStruct { .. } { }`
needs disambiguating parentheses in condition position — the parser would
otherwise try to read the struct literal as the `if`'s block.

## Basic usage example

```rust
let y = { // <- `{` opens a block expression
    let x = 1;
    x + 1 // no trailing `;`, so this is the block's value
}; // <- `}` closes it; y is now 2
```

## Embedded Rust Notes

**Full support.** Block and struct-literal delimiters are core grammar —
no `std` dependency.

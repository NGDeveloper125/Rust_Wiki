---
title: "try"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`try` was reserved specifically in the **2018 edition** — not 2015 like
most of this section's keywords. Before 2018, `try` was a perfectly
ordinary identifier; code from the 2015 edition that used `try` as a
variable or function name still compiles unchanged today, since edition
reservations are edition-scoped, not retroactive.

The reservation exists for **`try { }` blocks**: a proposed expression
form that would scope `?`-based error propagation to a block rather than
an entire function. Inside a `try { }` block, any `?` on a `Result`- or
`Option`-producing expression would short-circuit out of *the block*,
producing a `Result`/`Option` value for the block as a whole, instead of
returning early from the enclosing function the way `?` does everywhere
else today. This is real, fairly far-along design work — known
internally as "try blocks" or "catch expressions" — with an experimental
nightly implementation gated behind `#![feature(try_blocks)]`. It has
never stabilized, and open questions (mainly around type inference for
the block's error type) have kept it from progressing further, but this
is one of the more concretely developed reservations in this section.

It's worth being explicit about what this reservation does **not**
affect: the `TryFrom` and `TryInto` **traits** are ordinary, stable
library items in `std::convert` — fallible analogues of `From`/`Into`
used constantly in everyday Rust. They have nothing to do with the
reserved `try` keyword; a trait name and a keyword just happen to share
a root word.

Using `try` as an ordinary identifier is a compile error in the 2018
edition and later. The raw-identifier form `r#try` is legal in any
edition, the same escape hatch every reserved keyword offers.

## Basic usage example

```
let try = 5;     // error (2018 edition+): expected identifier, found reserved keyword `try`
let r#try = 5;   // ok: the raw-identifier form escapes the reservation
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

`try { }` would let a function scope error propagation to part of its
body instead of the whole thing. Today's real equivalent is pulling that
part out into its own small function or closure, purely so `?` has
somewhere narrower to return from.

```
fn process(raw: &str) -> String {
    // Today's real workaround: a nested fn/closure stands in for a
    // `try { }` block, giving `?` a narrower scope than the whole
    // outer function.
    let parsed: Result<u16, _> = (|| raw.trim().parse())();

    match parsed {
        Ok(port) => format!("port {port}"),
        Err(_) => "invalid port".to_string(),
    }
}
```

**Why this way:** without a stable `try { }` block, scoping `?` to less
than a whole function means introducing a helper closure or function
purely for that purpose — extra indirection that a `try { }` expression
would remove by letting `?` short-circuit out of the block itself
instead of the function; the
[unstable book's `try_blocks` entry](https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html)
documents the experimental syntax this reservation is held for.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

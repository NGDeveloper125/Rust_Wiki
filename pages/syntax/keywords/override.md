---
title: "override"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["final", "virtual"]
see_also: ["final", "virtual"]
---

## Explanation

`override` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
Like [`final`](final.md) and [`virtual`](virtual.md), it reads as a piece
of vocabulary from class-based inheritance (as in Java, C#, or C++,
where `override` marks a method that intentionally replaces a base
class's implementation). Rust has no such inheritance model — trait
implementations aren't "overridden," they're separate `impl` blocks — so
there is no concrete proposal attaching real behavior to `override`
today. It's speculative, held in reserve alongside `final` and `virtual`
in case some future mechanism gives Rust a reason to distinguish an
intentional override from an accidental name collision.

Using `override` as an ordinary identifier is a compile error today. The
raw-identifier form `r#override` is legal, the same escape hatch every
reserved keyword offers.

## Basic usage example

```
let override = 5;     // error: expected identifier, found reserved keyword `override`
let r#override = 5;   // ok: the raw-identifier form escapes the reservation
```

## Best practices & deeper information

There is no best-practice scenario to show here: `override` has no
function in today's Rust, and no concrete proposal to build one around,
so any "usage" example would be fiction. The one genuinely useful thing
to know is the raw-identifier escape hatch shown above.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

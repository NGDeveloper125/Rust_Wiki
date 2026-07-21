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

## Usage examples

### Using the raw-identifier escape hatch

```
let override = 5;     // error: expected identifier, found reserved keyword `override`
let r#override = 5;   // ok: the raw-identifier form escapes the reservation
```

## Explanation (Embedded)

**Full support.** Keyword reservation is a lexer-level fact, identical in
`#![no_std]` and hosted Rust alike — `override` carries no defined
meaning on any target, so there's no embedded-specific behavior to
describe.

## Usage examples (Embedded)

### The `override` reservation, unaffected by target

```
let override = 5;     // error: expected identifier, found reserved keyword `override`, on every target
let r#override = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

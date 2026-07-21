---
title: "final"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["override", "virtual"]
see_also: ["override", "virtual"]
---

## Explanation

`final` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
It's grouped here with [`override`](override.md) and [`virtual`](virtual.md)
because all three read like they'd belong to the same hypothetical
future feature: something analogous to `final` in Java or C++, a marker
that prevents further overriding or specialization of a trait
implementation or method. No concrete RFC exists for this today — it's
speculative, held in reserve in case Rust ever grows an
inheritance-like or specialization mechanism where "no further overrides
allowed" would be a meaningful thing to say.

Using `final` as an ordinary identifier is a compile error today. The
raw-identifier form `r#final` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### Escaping the reservation with a raw identifier

```
let final = 5;     // error: expected identifier, found reserved keyword `final`
let r#final = 5;   // ok: the raw-identifier form escapes the reservation
```

## Explanation (Embedded)

**Full support.** Keyword reservation is a lexer-level fact, identical in
`#![no_std]` and hosted Rust alike — `final` carries no defined meaning
on any target, so there's no embedded-specific behavior to describe.

## Usage examples (Embedded)

### The `final` reservation, unaffected by target

```
let final = 5;     // error: expected identifier, found reserved keyword `final`, on every target
let r#final = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

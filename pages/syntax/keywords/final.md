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

## Basic usage example

```
let final = 5;     // error: expected identifier, found reserved keyword `final`
let r#final = 5;   // ok: the raw-identifier form escapes the reservation
```

## Best practices & deeper information

There is no best-practice scenario to show here: `final` has no function
in today's Rust, and no concrete proposal to build one around, so any
"usage" example would be fiction. The one genuinely useful thing to know
is the raw-identifier escape hatch shown above.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

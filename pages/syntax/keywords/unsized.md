---
title: "unsized"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["?"]
see_also: ["?"]
---

## Explanation

`unsized` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
Its role is subtler than the other entries in this section: it isn't
pure speculation about some unbuilt feature so much as the unused flip
side of a mechanism that already shipped.

Every generic type parameter carries an implicit `Sized` bound by
default (`T` really means `T: Sized` unless said otherwise), because most
generic code needs to know its parameter's size at compile time. The
existing, working way to opt a parameter *out* of that implicit bound is
[`?Sized`](../operators/question-mark.md) — `T: ?Sized` relaxes the
default, allowing `T` to be a dynamically-sized type like `str` or
`dyn Trait`. A bare `unsized` keyword would presumably spell the same
relaxation a different way (`unsized T` instead of `T: ?Sized`), but
since `?Sized` already provides that escape hatch and has since before
1.0, there's no functional gap for a separate `unsized` keyword to fill.
This is likely why it has remained reserved-but-unclaimed rather than
becoming real syntax.

Using `unsized` as an ordinary identifier is a compile error today. The
raw-identifier form `r#unsized` is legal, the same escape hatch every
reserved keyword offers.

## Basic usage example

```
let unsized = 5;     // error: expected identifier, found reserved keyword `unsized`
let r#unsized = 5;   // ok: the raw-identifier form escapes the reservation
```

## Best practices & deeper information

### Scenario: Writing generic code

Accepting a dynamically-sized type behind a reference doesn't need the
reserved `unsized` keyword — the existing `?Sized` bound already relaxes
the implicit `Sized` requirement on a type parameter.

```
use std::fmt::Display;

fn describe<T: Display + ?Sized>(value: &T) -> String {
    // <- `?Sized` relaxes the implicit `Sized` bound; `unsized` plays no role here
    format!("value: {value}")
}

let via_str: &str = "reading";
println!("{}", describe(via_str)); // str is unsized; `?Sized` is what makes this compile
```

**Why this way:** without `?Sized`, `T: Display` alone would implicitly
require `T: Sized`, ruling out `str` and other dynamically-sized types
entirely — the
[Rust Reference on trait and lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html)
documents `?Sized` as the one relaxation the language allows on an
implicit default bound, which is exactly the gap a hypothetical
`unsized` keyword would otherwise need to fill.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

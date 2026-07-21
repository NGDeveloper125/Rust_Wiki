---
title: "box"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: ["Smart pointers (Box<T>)"]
related_syntax: []
see_also: []
---

## Explanation

`box` has been reserved since the 2015 edition, and — like [`priv`](priv.md)
— it has a real history rather than being purely speculative: for most of
Rust's pre-1.0 life and for years afterward, `box expr` was experimental,
nightly-only syntax for heap-allocating a value directly (`box 5` instead
of `Box::new(5)`), gated behind the unstable `box_syntax` feature, with a
matching `box pattern` form (`box_patterns`) for destructuring a `Box` by
value in a pattern. The appeal was **placement**: `box expr` could, in
principle, construct the value directly in its final heap location rather
than building it on the stack and then moving it into a freshly allocated
`Box` — avoiding a stack-to-heap copy for large values, something
`Box::new(expr)` cannot guarantee today (the compiler is free to elide
that copy as an optimization, but nothing in the language requires it to).

Both `box_syntax` and `box_patterns` were removed from nightly Rust in
2024 after years without progress toward stabilization — the placement-new
design space turned out to have unresolved questions (how it should
interact with fallible allocation, custom allocators, and `?` inside the
expression being boxed) that were never fully settled. `box` itself
remains reserved, keeping the door open for some future heap-construction
syntax, but there is no active nightly implementation using it today,
unlike [`become`](become.md) or [`gen`](gen.md).

Using `box` as an ordinary identifier is a compile error in every edition.
The raw-identifier form `r#box` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### Using the raw-identifier escape hatch

```
let box = 5;     // error: expected identifier, found reserved keyword `box`
let r#box = 5;   // ok: the raw-identifier form escapes the reservation
```

### Boxing and heap allocation

Today's real way to heap-allocate a value is the ordinary `Box::new`
associated function — no reserved keyword involved.

```
struct SensorFrame {
    readings: [f64; 64], // large enough that boxing it avoids a big stack copy
}

let frame = Box::new(SensorFrame { readings: [0.0; 64] }); // <- today's real heap-allocation syntax
```

`Box::new` is a perfectly ordinary function call, not
special syntax, and the compiler frequently optimizes away the
intermediate stack copy in practice even without a language-level
placement guarantee — see [Smart pointers (Box\<T\>)](../../concepts/ownership-borrowing/smart-pointers-box.md)
for when reaching for `Box` is the right call in the first place; the
now-removed `box_syntax` experiment was specifically about making that
elision a guarantee rather than an optimizer best-effort, which remains
unresolved design space rather than a feature in progress today.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike. `Box` itself needs `alloc` (see
[Smart pointers (Box\<T\>)](../../concepts/ownership-borrowing/smart-pointers-box.md)'s
own Embedded Rust Notes) — the reserved `box` keyword has no runtime
footprint of its own either way.

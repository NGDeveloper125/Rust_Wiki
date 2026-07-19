---
title: ","
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: ["( )", "[ ]"]
see_also: ["( )"]
---

## Explanation

`,` separates elements in any list: function arguments (`f(a, b, c)`),
tuple elements (`(a, b, c)`), array/`Vec` elements (`[a, b, c]`), struct
fields (`Point { x, y }`), enum variant fields, generic parameters
(`Vec<K, V>`), and match arms in some macro contexts.

A trailing comma after the last element is allowed (and idiomatic in
multi-line lists — `rustfmt` adds it automatically) in every one of these
positions:

```
let v = vec![
    1,
    2,
    3,
];
```

The one place a single trailing comma is *required*, not just allowed, is
a one-element tuple: `(x,)` — without the comma, `(x)` is just a
parenthesized expression, not a tuple at all.

## Basic usage example

```
let point = (1, 2, 3);
//            ^     ^ each `,` separates one tuple element from the next
```

**Restriction:** in a one-element tuple, the trailing comma is
*mandatory* — `(x,)` is a tuple, `(x)` is just `x` in parentheses.

## Embedded Rust Notes

**Full support.** Pure list-separator grammar — no `std` dependency.

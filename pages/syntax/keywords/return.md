---
title: "return"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Functions, Expression-oriented language]
related_syntax: [fn]
see_also: [fn]
---

## Explanation

`return` exits a function immediately with a given value:

```
fn abs(x: i32) -> i32 {
    if x < 0 {
        return -x;
    }
    x
}
```

Because Rust is expression-oriented, `return` is rarely required for the
*final* value of a function — the last expression in the body (with no
trailing semicolon) is returned implicitly. `return` exists for **early**
returns, typically from inside a conditional or loop, where control needs
to leave the function before reaching its end.

`return;` with no value is shorthand for `return ();` and is only valid
when the function's return type is `()`. `return` is itself an expression
of type `!` (never) — it never evaluates to anything at its own call
site, because control has already left — which lets it appear in
expression position, e.g. `let x = if cond { return; } else { 5 };`.

## Embedded Rust Notes

**Full support.** No `std` dependency. Note that a `#![no_std]` binary's
`fn main() -> !` never returns at all — `return` is used for early exits
from ordinary functions, same as on a hosted target.

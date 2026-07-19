---
title: "break"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: [loop, while, for, continue]
see_also: [continue, loop]
---

## Explanation

`break` exits the nearest enclosing loop immediately:

```
loop {
    if done {
        break;
    }
}
```

Inside a `loop` (but not `while`/`for`), `break` can carry a value —
`break value;` — which becomes the result of the whole `loop` expression.
This is the *only* loop form where that's legal, since `while`/`for` may
execute zero iterations and therefore can't guarantee a value exists to
break with.

To exit an outer loop from inside a nested one, label the outer loop and
target it explicitly: `break 'outer;` (see loop labels under `loop`,
`while`, `for`). `break` can also be used inside a labeled non-loop block
(`'a: { ... break 'a value; }`) to exit early with a value, a lesser-known
form useful for structuring multi-step logic without a `loop` at all.

## Embedded Rust Notes

**Full support.** No `std` dependency; works identically in `#![no_std]`.

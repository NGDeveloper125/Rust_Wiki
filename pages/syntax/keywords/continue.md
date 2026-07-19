---
title: "continue"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [loop, while, for, break]
see_also: [break]
---

## Explanation

`continue` skips the rest of the current loop iteration's body and jumps
straight to the next iteration's condition check (for `while`/`for`) or
back to the top of the body (for `loop`):

```
for x in 0..10 {
    if x % 2 == 0 {
        continue;
    }
    println!("{x}"); // only odd numbers
}
```

`continue` never carries a value — unlike `break`, there is no
`continue value;` form, since "continuing" doesn't produce a result the
way exiting a `loop` can. Like `break`, it can target a labeled outer loop
explicitly with `continue 'label;` from inside a nested loop.

## Embedded Rust Notes

**Full support.** No `std` dependency; works identically in `#![no_std]`.

---
title: "while"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership]
related_syntax: [loop, for, break, continue]
see_also: [loop, for]
---

## Explanation

`while` repeats a block for as long as a condition remains `true`:

```
while count < 10 {
    count += 1;
}
```

Like `if`, the condition must be a plain `bool` and is not parenthesized
by convention. Unlike `if`, `while` is **not** an expression — it always
evaluates to `()` and cannot be used to produce a value, because the loop
body may run zero times, leaving no meaningful value to yield (contrast
with `loop`, which can `break` with a value precisely because it's
guaranteed to run its body evaluation at least once before exiting via
`break`).

`while let PATTERN = expr { ... }` is a related but distinct form: it
loops for as long as `expr` continues to match `PATTERN`, re-evaluating
`expr` and testing the match on every iteration — commonly used to drain
an iterator or a channel one item at a time.

A `while` loop can be given a label (`'outer: while ... `) so an inner
`break` or `continue` can target it specifically instead of the nearest
enclosing loop.

## Embedded Rust Notes

**Full support.** `while` polling loops are a staple of bare-metal
embedded code (spinning on a status register bit until a peripheral is
ready) when interrupts aren't used instead.

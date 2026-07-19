---
title: "loop"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: [while, for, break, continue]
see_also: [while, break]
---

## Explanation

`loop` repeats a block unconditionally, forever, until a `break` inside it
(or its body) is reached:

```
loop {
    if done {
        break;
    }
}
```

Unlike `while` and `for`, `loop` **is** an expression that can produce a
value — `break value;` exits the loop and evaluates the whole `loop` to
`value`:

```
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;
    }
};
```

This is possible precisely because the compiler can see there's no
"falling off the end without a value" case to reconcile, the way there is
with `while`/`for` (which might run zero iterations). A `loop` with no
`break` at all has type `!` (never) — the compiler knows control can never
leave it normally, which is useful for things like a server's main event
loop.

Like other loops, `loop` accepts a label (`'a: loop { ... }`) so nested
`break`/`continue` can target a specific enclosing loop.

## Embedded Rust Notes

**Full support.** An unconditional `loop {}` is the idiomatic body of
`fn main() -> !` in almost every embedded firmware entry point, since a
bare-metal program never "exits" the way a hosted process does.

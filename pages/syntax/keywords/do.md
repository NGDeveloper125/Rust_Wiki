---
title: "do"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`do` has been reserved since the 2015 edition, as part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
There is no strong, publicly documented direction for what it will
eventually become. The most commonly guessed future is some form of a
`do`-`while` loop (a loop that runs its body once before checking the
condition, as in C and several other languages) — Rust today has no such
form; `loop { ... }` with a manual `break` is the closest equivalent. Be
honest that this is speculative: no active RFC is attached to `do`, and
it may simply remain unclaimed indefinitely.

Using `do` as an ordinary identifier is a compile error today. The
raw-identifier form `r#do` is legal, the same escape hatch every reserved
keyword offers.

## Basic usage example

```
let do = 5;     // error: expected identifier, found reserved keyword `do`
let r#do = 5;   // ok: the raw-identifier form escapes the reservation
```

## Best practices & deeper information

There is no best-practice scenario to show here: `do` has no function in
today's Rust, and no concrete proposal to build one around, so any
"usage" example would be fiction. The one genuinely useful thing to know
is the raw-identifier escape hatch shown above.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

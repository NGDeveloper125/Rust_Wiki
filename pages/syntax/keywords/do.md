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

## Usage examples

### Using the raw-identifier escape hatch

```
let do = 5;     // error: expected identifier, found reserved keyword `do`
let r#do = 5;   // ok: the raw-identifier form escapes the reservation
```

## Explanation (Embedded)

**Full support.** Keyword reservation is a lexer-level fact, identical in
`#![no_std]` and hosted Rust alike — `do` carries no defined meaning on
any target, so there's no embedded-specific behavior to describe.

## Usage examples (Embedded)

### The `do` reservation, unaffected by target

```
let do = 5;     // error: expected identifier, found reserved keyword `do`, on every target
let r#do = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

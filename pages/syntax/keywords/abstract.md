---
title: "abstract"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`abstract` has been reserved since the 2015 edition — it's part of the
original batch of reserved words the
[Rust Reference's keyword list](https://doc.rust-lang.org/reference/keywords.html)
carried from Rust's early design period, before 1.0. Being reserved means
the lexer recognizes `abstract` as a keyword token even though no grammar
rule currently gives it any meaning: you cannot use it as the name of a
variable, function, type, or anything else.

Reserving it kept a door open for a possible future "abstract" modifier on
types or methods, in the spirit of abstract classes in Java or C++ — a
declaration that states a member exists without providing (or requiring at
that site) an implementation. Be honest about where things stand: there is
no concrete RFC actively progressing this. `abstract` is best described as
a long-standing placeholder inherited from Rust's early keyword list
rather than an active roadmap item — unlike [`become`](become.md) or
[`gen`](gen.md), which do have live design efforts behind them.

Trying to use `abstract` as an ordinary identifier is a compile error in
every edition. The escape hatch is the **raw identifier** form,
`r#abstract` — raw-identifier syntax lets any keyword, including a
reserved one with no defined meaning at all, be used as a plain name when
a real name genuinely collides with it (for example, code generated from,
or bound to, another language's API that happens to use the word
`abstract`).

## Usage examples

### Using the raw-identifier escape hatch

```
let abstract = 5;     // error: expected identifier, found reserved keyword `abstract`
let r#abstract = 5;   // ok: the raw-identifier form escapes the reservation
```

## Explanation (Embedded)

**Full support.** Keyword reservation is a lexer-level fact, identical in
`#![no_std]` and hosted Rust alike — `abstract` carries no defined
meaning on any target, so there's no embedded-specific behavior to
describe.

## Usage examples (Embedded)

### The `abstract` reservation, unaffected by target

```
let abstract = 5;     // error: expected identifier, found reserved keyword `abstract`, on every target
let r#abstract = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

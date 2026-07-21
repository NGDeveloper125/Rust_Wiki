---
title: "yield"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["gen"]
see_also: ["gen"]
---

## Explanation

`yield` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
It's closely tied to — and likely to be subsumed by — the newer
[`gen`](gen.md) reservation from the 2024 edition: `yield value` is the
piece inside a hypothetical `gen fn`/`gen { }` block that would suspend
execution and produce an intermediate value, resuming from that exact
point on the next request for a value. The relationship mirrors
`.await` inside `async fn`: `.await` suspends waiting *for* a value that
isn't ready; `yield` would suspend right *after producing* a value,
handing it out before continuing.

Like `gen`, this has genuine design momentum behind it — the same
experimental nightly implementation gated behind
`#![feature(gen_blocks)]` that supports `gen { }` blocks also supports
`yield` inside them, on nightly Rust today. Nothing here is stable, and
the exact syntax could still change, but `yield` is one of this
section's more concretely-motivated reservations precisely because it's
paired with `gen`'s active work rather than standing alone.

Using `yield` as an ordinary identifier is a compile error today. The
raw-identifier form `r#yield` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### The `yield` reservation error and raw-identifier escape hatch

```
let yield = 5;     // error: expected identifier, found reserved keyword `yield`
let r#yield = 5;   // ok: the raw-identifier form escapes the reservation
```

### Working with collections

A `gen`/`yield` block would let you write a sequence-producing function
that looks like ordinary sequential code with suspension points, instead
of the state machine you have to hand-write today by implementing
`Iterator` directly.

```
struct Countdown(u32);

// Today's real, stable equivalent: a manual Iterator implementation
// stands in for what a `gen fn` with `yield` statements would express
// more directly — this `next` call is where a `yield value` would sit.
impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

let values: Vec<u32> = Countdown(3).collect();
```

Implementing `Iterator` by hand means manually
tracking, in `self`, exactly where the last call left off — the state a
`gen fn` would instead capture automatically at each `yield` point, the
same way `async fn` automatically captures where execution paused at
each `.await`.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

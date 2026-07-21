---
title: "gen"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["yield", "async"]
see_also: ["yield", "async"]
---

## Explanation

`gen` is the newest reservation in this group: it became a reserved
keyword only in the **2024 edition**, unlike the rest of this section's
keywords, most of which have been reserved since 2015. That recency
matches how recent the design work behind it is.

`gen` is earmarked for a possible future **generator/coroutine** syntax —
`gen fn` or `gen { }` blocks that produce a sequence of values one at a
time via [`yield`](yield.md), the same way `async fn` produces a
`Future` that resolves via `.await`. The analogy is close and
deliberate: `async fn` already desugars into a state machine that
implements `Future`; a `gen fn` would desugar into a state machine that
implements `Iterator` instead. Where `async` suspends waiting for a
value that isn't ready yet, `gen` would suspend *after* producing a
value, resuming from that exact point on the next call for the next one.

This has real, relatively recent design momentum: there is an
experimental nightly implementation gated behind
`#![feature(gen_blocks)]` that supports `gen { }` block syntax today on
nightly Rust, and it's this active work that justifies `gen` being
reserved specifically in 2024 rather than having sat unclaimed since
2015 like [`abstract`](abstract.md) or [`do`](do.md). None of this is
stable yet, and reservation doesn't guarantee eventual stabilization —
but of this section's keywords, `gen` and [`become`](become.md) are the
two with genuine forward motion behind them.

Using `gen` as an ordinary identifier is a compile error in the 2024
edition (and later); it was never reserved in 2015/2018/2021, so
existing code using `gen` as a variable or function name from those
editions is unaffected until it migrates. The raw-identifier form
`r#gen` is legal in any edition, the same escape hatch every reserved
keyword offers.

## Usage examples

### Escaping the reservation with a raw identifier

```
let gen = 5;     // error (2024 edition): expected identifier, found reserved keyword `gen`
let r#gen = 5;   // ok: the raw-identifier form escapes the reservation
```

### Async tasks

`async fn` is the closest thing Rust has today to what `gen fn` would
be — both desugar a function body with suspension points into a state
machine, just implementing a different trait at the end.

```
// Today's real, stable parallel: an async fn suspends at `.await`
// points and produces a `Future`; a future `gen fn` would suspend at
// `yield` points and produce an `Iterator` instead — same shape,
// different trait.
async fn fetch_count(id: u32) -> u32 {
    // suspends here, waiting for a value that isn't ready yet
    lookup(id).await
}

async fn lookup(id: u32) -> u32 {
    id * 2
}
```

Understanding `gen` is easiest by analogy to
`async`/`.await`, which already ships: both are ways of writing a
sequential-looking function body that the compiler transforms into a
resumable state machine, differing only in what triggers a suspension
(awaiting a future vs. yielding a value) and which trait the result
implements (`Future` vs. `Iterator`).

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

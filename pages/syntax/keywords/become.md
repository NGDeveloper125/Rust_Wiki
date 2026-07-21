---
title: "become"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`become` has been reserved since the 2015 edition, but unlike most of its
neighbors in the
[Rust Reference's reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html),
it has genuine forward momentum: it's earmarked for **guaranteed
tail-call elimination**. The idea is `become f(x)` as an explicit,
distinct alternative to `return f(x)` — one that promises the compiler
will reuse the current stack frame for the call to `f` rather than
pushing a new one, so a chain of `become` calls runs in constant stack
space no matter how deep the logical recursion goes. A plain
`return f(x)` never carries that guarantee today; the compiler *may*
perform tail-call optimization as an implementation detail, but nothing
in the language lets you require or rely on it.

This is an active area of work: there is an experimental nightly
implementation gated behind `#![feature(explicit_tail_calls)]` that uses
exactly this `become` syntax, developed alongside ongoing design
discussion about the guarantees such a construct would need (matching
argument/return types between caller and callee, restrictions inside
`unsafe` blocks with live destructors, and so on). None of this is stable,
and the shape could still change before (if) it stabilizes — but this
reservation is much closer to "future feature in progress" than the
purely speculative entries like [`do`](do.md) or [`final`](final.md).

Using `become` as an ordinary identifier is a compile error today. The
raw-identifier form `r#become` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### Using the raw-identifier escape hatch

```
let become = 5;     // error: expected identifier, found reserved keyword `become`
let r#become = 5;   // ok: the raw-identifier form escapes the reservation
```

### Boxing and heap allocation

A recursive function walking a linked structure is exactly the case
`become` targets: written naturally, each recursive call adds a stack
frame, so a long enough chain risks a stack overflow. Today's real
workaround is to rewrite the recursion as an explicit loop.

```
struct Node {
    value: i64,
    next: Option<Box<Node>>,
}

// Today's real alternative: an explicit loop, not recursion.
fn sum(mut node: Option<&Node>) -> i64 {
    let mut total = 0;
    while let Some(n) = node {
        total += n.value;
        node = n.next.as_deref();
    }
    total
}
```

Without a stable tail-call guarantee, a recursive
version of `sum` (`n.value + sum(n.next.as_deref())`) has no promise of
constant stack usage, so idiomatic Rust today reaches for the loop form
instead of trusting the optimizer; `become`, if stabilized, would let the
recursive form itself carry that guarantee instead of requiring the
manual rewrite.

## Explanation (Embedded)

**Full support.** Keyword reservation itself is a lexer-level fact,
identical in `#![no_std]` and hosted Rust alike. The feature `become` is
reserved for — guaranteed tail-call elimination via
`#![feature(explicit_tail_calls)]` — is equally unstable on every target
today, so there's nothing embedded-specific to demonstrate yet. It's
worth naming why this reservation is worth watching more than most of
its neighbors in this group, though: embedded targets typically run with
a small, fixed-size stack and no virtual memory to grow into, so a
stable, guaranteed constant-stack-space tail call would matter more there
than on a hosted target with megabytes of stack and an OS underneath it.
That's a reason to care if/when it stabilizes, not a difference in
today's behavior — the reservation and the missing feature are identical
right now regardless of target.

## Usage examples (Embedded)

### The `become` reservation, unaffected by target

```
let become = 5;     // error: expected identifier, found reserved keyword `become`, on every target
let r#become = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

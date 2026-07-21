---
title: "#![feature(...)]"
kind: attribute
embedded_support: full
groups: ["Compiler Hints & Limits", "Memory & Unsafe"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`#![feature(name)]`, an inner attribute at a crate root, opts the crate
into an **unstable** compiler or language feature named `name` — some
piece of not-yet-stabilized syntax, standard library API, or compiler
capability that exists in the compiler's source but isn't part of stable
Rust. Writing `#![feature(...)]` and building with the stable (or beta)
toolchain is a **hard compile error** — this attribute only works at all
on the **nightly** toolchain; it's the mechanism nightly-only Rust
actually *is*, syntactically. There is no equivalent for stable code:
either a capability has been stabilized, and needs no `#![feature(...)]`
at all, or it hasn't, and using it requires nightly plus this attribute
naming it explicitly.

This exists so the language and standard library can experiment: new
syntax, new trait machinery, or a new `core`/`std` API can be implemented
and used by real projects on nightly, gathering feedback, well before
anyone commits to the (much harder to walk back) guarantee that comes with
stabilizing it for stable Rust. A crate that uses `#![feature(...)]` is
making a deliberate trade: access to capability that doesn't exist any
other way yet, in exchange for opting out of Rust's usual stability
guarantee for that specific piece of the language.

**Here be dragons — read this before reaching for it.** An unstable
feature can change shape, gain new restrictions, or be removed outright
between nightly releases without the deprecation cycle a stable API would
get; code depending on one can simply stop compiling on a newer nightly
with no advance warning. This makes `#![feature(...)]` fundamentally
different from every other attribute on this wiki: it isn't just
"advanced" or "niche," it's an explicit acknowledgment that the ground
underneath the annotated crate can shift at any time, which is why it's
essentially never seen in a published, stable-toolchain-targeting crate —
almost every real-world user is either the standard library itself,
compiler tooling, or a project that has specifically chosen to track
nightly.

## Usage examples

### Enabling an unstable language feature on nightly

```
// Nightly-only — this attribute and the feature it names are not available on stable/beta.
#![feature(let_chains)]

fn describe(value: Option<i32>) -> &'static str {
    if let Some(n) = value && n > 0 {
        "positive"
    } else {
        "non-positive or absent"
    }
}
```

### Designing a public API

A project deliberately tracks nightly Rust to use an unstabilized
standard library API not available any other way, accepting the risk that
a future nightly could change or remove it — framed honestly as a
nightly-only choice, not something expected to reach every user's stable
toolchain.

```
// Nightly-only. On stable/beta this line alone is a compile error naming
// the unknown feature; there is no stable equivalent to fall back to.
#![feature(step_trait)]

use std::iter::Step;

// Real usage would implement `Step` for a custom type to make it usable
// directly in a range expression (`MyType(0)..MyType(10)`) — an
// unstabilized capability at the time of writing, illustrative here.
```

Because unstable features carry no compatibility
guarantee between nightly releases, a project reaching for
`#![feature(...)]` is making a considered, ongoing maintenance commitment
— tracking nightly, watching for breaking changes to the specific feature
used, and being ready to adapt or drop the feature if it's altered before
stabilizing; the
[Unstable Book](https://doc.rust-lang.org/unstable-book/) documents each
feature gate along with its current status for exactly this kind of
ongoing risk assessment before depending on one.

## Embedded Rust Notes

**Full support**, in the sense that `#![feature(...)]` itself works
identically regardless of `std`/`#![no_std]` — it's a compiler-toolchain
gate, not a standard-library one. Worth noting for embedded specifically:
historically, several capabilities embedded projects wanted badly (const
generics, certain `core` APIs) spent time as nightly-only features before
stabilizing, so embedded codebases have, in practice, been more likely
than most to track nightly and use `#![feature(...)]` while waiting for a
needed capability to stabilize — still subject to the same "this can
change without notice" caveat as anywhere else.

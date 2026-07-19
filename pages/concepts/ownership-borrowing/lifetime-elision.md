---
title: "Lifetime elision"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Lifetime Management"]
related_syntax: ["'ident"]
see_also: ["Lifetimes"]
---

## Explanation

Most function signatures involving references never need an explicit
lifetime annotation at all, because the compiler applies a fixed set of
elision rules to infer them automatically. Roughly: each elided input
reference gets its own lifetime; if there's exactly one input lifetime,
it's assigned to every elided output lifetime; and if one of the inputs
is `&self`/`&mut self`, its lifetime is assigned to the elided outputs.

```
fn first_word(s: &str) -> &str { ... } // no lifetimes written,
                                        // but fully well-defined
```

This exists purely for ergonomics — the underlying rule (a returned
reference can't outlive its source) is exactly as strict whether or not
you write `'a` explicitly. Elision only kicks in for the common,
unambiguous patterns the rules cover; anything the rules can't resolve
(most commonly, multiple input references where the output could plausibly
relate to either) requires spelling the lifetime out by hand, precisely
because the compiler has no safe default to guess.

## Basic usage example

```
struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &str { // <- elided: output borrows from `&self`, inferred automatically
        self.input
    }
}
```

## Embedded Rust Notes

**Full support.** Same elision rules apply regardless of target — no
`std` dependency.

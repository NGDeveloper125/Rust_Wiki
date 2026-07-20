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
For instance, `fn first_word(s: &str) -> &str` needs no lifetime written
at all — with exactly one input reference, its lifetime is assigned to
the elided output automatically, and the signature is fully well-defined.

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

## Best practices & deeper information

### Scenario: Designing a public API

A `Cache`'s lookup method relies entirely on elision — with exactly one
reference-typed receiver (`&self`), its lifetime is silently assigned to
the returned reference, so the signature stays clean.

```
struct Cache {
    entries: Vec<(String, String)>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<&str> { // <- elided: return borrows from `&self`, not `key`
        self.entries.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}
```

**Why this way:** because there's exactly one reference-typed receiver,
the elision rules assign its lifetime to the elided output automatically
— writing `fn get<'a>(&'a self, key: &str) -> Option<&'a str>` by hand
would express the identical contract with no added clarity, which is why
the
[Rust Book](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision)
recommends leaving cases like this elided; see [Lifetimes](lifetimes.md)
for when spelling one out by hand is actually necessary.

## Embedded Rust Notes

**Full support.** Same elision rules apply regardless of target — no
`std` dependency.

---
title: "Lifetime elision"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Lifetime Management"]
related_syntax: ["'a"]
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

## Explanation (Embedded)

Lifetime elision is a compile-time inference rule with no representation
in the compiled output at all, so it applies identically no matter what
target the code is built for — there's no `no_std`-specific elision rule,
and nothing about a `#![no_std]` crate changes which signatures qualify
for it. The syntax-level mechanics of writing an explicit lifetime by hand
once elision doesn't apply are covered from the token's own angle by
['a (named lifetime)](../../syntax/lifetimes/named-lifetime.md) and, for
the program-long case, [`'static`](../../syntax/lifetimes/static-lifetime.md)
— this page's embedded angle is the design question elision raises rather
than answers: because elision only ever produces the lifetime a signature
would need to write out explicitly anyway, leaning on it is never a
shortcut that papers over a real embedded-specific constraint. A `&self`
method on an embedded driver that returns a borrowed reading gets its
output lifetime elided exactly as readily as any hosted method would —
borrowing a peripheral handle or reading a register doesn't change the
elision rules themselves, only the fraction of real embedded signatures
that end up needing an explicit lifetime at all (see
[Lifetimes (Embedded)](lifetimes.md) for why that fraction runs somewhat
higher than on hosted code, mainly around `'static` and driver structs
that borrow rather than own).

## Basic usage example (Embedded)

```
struct Log<'a> {
    entries: &'a [u8],
}

impl<'a> Log<'a> {
    fn tail(&self) -> &[u8] { // <- elided: output borrows from `&self`, inferred automatically
        &self.entries[self.entries.len().saturating_sub(4)..]
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A sensor driver's history-buffer lookup relies entirely on elision — with
one reference-typed receiver (`&self`), its lifetime is silently assigned
to the returned reference, keeping the signature as clean as the
equivalent hosted API would be.

```
struct ReadingHistory {
    samples: heapless::Vec<i16, 16>,
}

impl ReadingHistory {
    fn latest(&self) -> Option<&i16> { // <- elided: return borrows from `&self`, not from any other input
        self.samples.last()
    }
}
```

**Why this way:** exactly one reference-typed receiver means the elision
rules assign its lifetime to the elided output automatically, whether the
surrounding crate is `std` or `#![no_std]` — writing `fn latest<'a>(&'a
self) -> Option<&'a i16>` by hand would express the identical contract
with no added clarity, the same tradeoff the
[Rust Book](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#lifetime-elision)
describes for the hosted case; see [Lifetimes (Embedded)](lifetimes.md)
for when an embedded driver signature actually does need a lifetime
spelled out by hand.

---
title: "Lifetimes"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Lifetime Management", "Unique to Rust"]
related_syntax: ["'a", "&"]
see_also: ["The borrow checker", "Lifetime elision", "Borrowing (shared references)"]
---

## Explanation

A lifetime describes how long a reference remains valid — specifically,
that it cannot outlive the value it points to. Every reference in Rust
has a lifetime, whether or not it's written out explicitly; naming it
(`'a`) becomes necessary only when the compiler can't work out on its own
that a function's inputs and outputs relate correctly.

The concept lifetimes exist to express is straightforward: `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`
says "the reference this function returns lives no longer than the
shorter-lived of the two inputs" — a real constraint on the function's
contract, not a formality. Without that annotation, the compiler would
have no way to verify the caller isn't left holding a dangling reference
after the inputs it borrowed from go out of scope.

Lifetimes are a **compile-time-only** concept — they exist purely to let
the borrow checker prove reference validity ahead of time, and are erased
entirely before the program runs (there is no runtime lifetime tracking,
no reference counting implied by writing `'a`). This is precisely why
Rust can guarantee no dangling references with zero runtime cost: the
proof happens once, at compile time, instead of via a runtime check
(garbage collection, reference counting) every single language with
manual memory management otherwise needs.

## Basic usage example

```
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { // <- ties the output's lifetime to both inputs
    if x.len() > y.len() { x } else { y }
}

let s1 = String::from("long string");
let result;
{
    let s2 = String::from("short");
    result = longest(&s1, &s2);
    println!("{result}"); // must be used while s2 is still alive
}
```

**Restriction:** the returned reference can't outlive the shorter-lived
input — using `result` after `s2` goes out of scope would fail to
compile, since `'a` is bound by the shorter of the two borrows.

## Best practices & deeper information

### Scenario: Designing a public API

A `Parser`'s methods should carry an explicit lifetime parameter only
where the elision rules genuinely can't infer one — anywhere they can,
writing `'a` out by hand adds noise without adding a constraint.

```
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    // AVOID: writing out a lifetime the elision rules already infer for free
    // fn peek<'b>(&'b self) -> Option<&'b str> { self.input.get(self.pos..) }

    fn peek(&self) -> Option<&str> { // <- PREFER: elided; output borrows from `&self` automatically
        self.input.get(self.pos..)
    }
}
```

**Why this way:** an explicit lifetime that elision would have inferred
anyway doesn't express anything the compiler didn't already know — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/) favor the
simplest signature that expresses the real contract, which keeps a
type's explicit lifetime parameters meaningful on the cases where they
truly are load-bearing (see [Lifetime elision](lifetime-elision.md)).

### Scenario: Sharing data with multiple references

A function borrowing from two differently-lived inputs needs its
lifetime annotation to be what lets the compiler catch a caller trying to
use the result after the shorter-lived input is gone.

```
fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str { // <- ties the result to whichever input's borrow ends first
    if a.len() < b.len() { a } else { b }
}

let long_lived = String::from("configuration");
let result;
{
    let short_lived = String::from("cfg");
    result = shorter(&long_lived, &short_lived);
    println!("{result}"); // must run while short_lived is still alive
}
// using `result` here would fail to compile: it may borrow from short_lived, now dropped
```

**Why this way:** naming the shared lifetime `'a` across both parameters
and the return type is what lets the borrow checker reject uses of
`result` after the shorter-lived input is gone — without it, the compiler
would otherwise have to assume the return value could outlive either
input, which the
[Rust Book](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
covers as the core case explicit lifetimes exist for.

## Embedded Rust Notes

**Full support.** Lifetimes are erased entirely before codegen — a purely
compile-time concept with zero runtime footprint, identical on every
target including `#![no_std]`.

---
title: "Anti-pattern: Deref polymorphism (faking inheritance)"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Composition"]
related_syntax: ["*"]
see_also: ["Deref & DerefMut coercion", "Traits", "Dependency injection via traits/generics"]
---

## Explanation

[Deref & DerefMut coercion](../ownership-borrowing/deref-coercion.md)
covers what `Deref` is *for*: letting a smart pointer transparently act
like a reference to whatever it wraps, so `Box<String>` can be used
almost anywhere a `&String` is expected. This page is about a specific
misuse of that mechanism: implementing `Deref` on a wrapper type not to
make it act like a reference, but purely to make the wrapper "inherit"
every method of the type it wraps — a `Wrapper<Inner>` that implements
`Deref<Target = Inner>` so `wrapper.some_inner_method()` compiles via
autoderef, imitating the way a subclass in an object-oriented language
inherits its parent's methods.

It's tempting because it genuinely works and requires almost no code: one
`impl Deref for Wrapper` block, and every public method of `Inner`
suddenly appears to be a method of `Wrapper` too, without writing a
single forwarding function by hand. For someone thinking in terms of
class inheritance, it looks like exactly the tool for the job.

The trouble is that `Deref` was never designed to mean "is-a" — it means
"acts like a reference to." Stretching it to fake inheritance creates
real problems: every method of `Inner` leaks into `Wrapper`'s public
surface whether or not that's actually wanted, so the wrapper's true API
becomes whatever `Inner` happens to expose, plus whatever methods
`Wrapper` adds — which can silently shadow an `Inner` method of the same
name in confusing ways. It also breaks down the moment `Wrapper` legitimately
needs to diverge from `Inner`'s behavior for one of those methods,
because autoderef doesn't know to prefer `Wrapper`'s own logic over a
method resolution that quietly reaches through to `Inner`.

The idiomatic alternative is explicit delegation: write forwarding
methods by hand for exactly the operations `Wrapper` intends to expose
(a handful of lines, ideally with each doing real work — validating,
logging, adjusting an argument — rather than being pure boilerplate), or
have both `Wrapper` and `Inner` implement a shared
[trait](../traits-polymorphism/traits.md) that names the behavior they
have in common. Either way, `Wrapper`'s public API is exactly what its
author wrote, not an accidental byproduct of what `Inner` happens to
expose — composition, not inherited method access via a pointer
coercion.

## Basic usage example

```
struct Meters(f64);

impl Meters {
    fn to_feet(&self) -> f64 { // <- PREFER: an explicit method, not an inherited f64 method via Deref
        self.0 * 3.28084
    }
}

let distance = Meters(10.0);
println!("{:.2} ft", distance.to_feet());
```

## Best practices & deeper information

### Scenario: Designing a public API

A logging wrapper around a database connection wants callers to run
queries through it (so it can log them), not bypass logging by reaching
straight through to every method the inner connection happens to expose.

```
struct DbConnection;

impl DbConnection {
    fn query(&self, sql: &str) -> Vec<String> {
        vec![format!("row for: {sql}")]
    }

    fn execute(&self, sql: &str) {
        println!("executed: {sql}");
    }
}

// AVOID: Deref makes every DbConnection method available on LoggingConnection unintentionally
// struct LoggingConnection {
//     inner: DbConnection,
// }
//
// impl std::ops::Deref for LoggingConnection {
//     type Target = DbConnection;
//     fn deref(&self) -> &DbConnection {
//         &self.inner // now `logging_conn.execute(..)` silently skips logging entirely
//     }
// }

// PREFER: explicit delegation — only the methods the wrapper actually intends to expose, each doing its own job
struct LoggingConnection {
    inner: DbConnection,
}

impl LoggingConnection {
    fn query(&self, sql: &str) -> Vec<String> {
        println!("[log] query: {sql}");
        self.inner.query(sql) // <- forwards deliberately, with logging attached
    }

    fn execute(&self, sql: &str) {
        println!("[log] execute: {sql}");
        self.inner.execute(sql);
    }
}

let conn = LoggingConnection { inner: DbConnection };
conn.query("SELECT 1"); // always logged: there's no bypass path through Deref
```

**Why this way:** implementing `Deref` on `LoggingConnection` would make
`execute` (and any future `DbConnection` method) reachable straight
through the wrapper without ever going through the wrapper's own logic,
silently defeating the entire point of wrapping it; the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html)
documents "Deref polymorphism" as exactly this misuse, and recommends
explicit delegation or a shared trait instead.

## Embedded Rust Notes

**Full support.** Both the anti-pattern and its fix are pure trait-impl /
method-resolution concerns handled entirely at compile time, with no
runtime cost either way under `#![no_std]` — the guidance to prefer
explicit delegation over faked inheritance applies identically regardless
of target.

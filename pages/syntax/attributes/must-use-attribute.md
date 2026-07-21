---
title: "#[must_use]"
kind: attribute
embedded_support: full
groups: ["Design Patterns & Idioms"]
related_concepts: []
related_syntax: ["_"]
see_also: ["_"]
---

## Explanation

`#[must_use]` placed on a **function** (or method) warns if the caller
discards its return value without using it in any way — binding it to a
name that's later read, passing it onward, or otherwise doing something
with it. Simply calling the function as a bare statement and letting the
result evaporate triggers the warning. This is exactly the mechanism
behind one of the standard library's most-seen warnings: `Result` itself
is `#[must_use]`, which is why calling a `Result`-returning function and
ignoring the result — never checking whether it was `Ok` or `Err` — warns
by default, since a silently-ignored error is very often a real bug.

`#[must_use]` placed on a **type definition** (a struct or enum) instead
propagates the same warning to *every* function anywhere that returns that
type — there's no need to separately mark each function; the attribute on
the type is enough. This is exactly how `Result` gets its behavior: the
`Result` type itself is `#[must_use]`, not each individual function that
happens to return one.

The optional `#[must_use = "explanation"]` form attaches a custom message
shown alongside the warning, in place of the generic "unused value that
must be used" text — useful for saying specifically *why* discarding this
particular value is probably a mistake, e.g.
`#[must_use = "this returns a new iterator rather than modifying the original"]`,
a message the standard library itself uses on adapters like
`Iterator::map`.

Discarding a `#[must_use]` value is sometimes genuinely intentional — a
best-effort cleanup whose outcome truly doesn't matter to the caller. The
idiomatic way to silence the warning in that specific case is
`let _ = expression;`, covered in depth (including why it differs from a
bare statement) on the [`_`](../punctuation/underscore.md) page — reach
for it deliberately, at the one call site where ignoring really is
correct, rather than removing `#[must_use]` from the function or type
itself.

## Basic usage example

```
#[must_use] // <- warns if the caller doesn't do anything with the returned value
fn discount_applied(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}

fn main() {
    let final_price = discount_applied(1000, 10); // used: no warning
    println!("{final_price}");
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A builder's setter methods each return `Self` to enable chaining — if a
caller calls one and discards the result instead of continuing the chain,
that call had no effect at all, which is exactly the kind of silent bug
`#[must_use]` is meant to catch.

```
#[must_use = "builder methods return a new builder; the original is not modified"] // <- custom message
pub struct RequestBuilder {
    timeout_ms: u32,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder { timeout_ms: 1000 }
    }

    pub fn timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

fn build_request() -> RequestBuilder {
    // AVOID: this call's return value is discarded — timeout_ms(5000) had no effect at all
    // RequestBuilder::new().timeout_ms(5000);

    // PREFER: the chain's final value is the one that's actually used
    RequestBuilder::new().timeout_ms(5000)
}
```

**Why this way:** `#[must_use]` on the type means every method that
returns `RequestBuilder` — not just `new`, every chained setter too — is
covered by one attribute, so discarding any link in the chain is flagged;
the custom message clarifies specifically *why* discarding it is
suspicious (a builder call is never mutating something in place), matching
how the standard library documents its own
[`#[must_use]` conventions](https://doc.rust-lang.org/std/result/enum.Result.html)
on `Result` and iterator adapters.

### Scenario: Handling and propagating errors

A function returning `Result` should let that `Result`'s own
`#[must_use]` do its job — deliberately discarding one specific,
known-safe-to-ignore call still needs an explicit `let _ = ...` rather
than restructuring the function to avoid the warning.

```
fn write_audit_log(message: &str) -> Result<(), std::io::Error> {
    println!("audit: {message}");
    Ok(())
}

fn shut_down() {
    let _ = write_audit_log("shutting down"); // <- deliberate: a failed audit log shouldn't block shutdown
}
```

**Why this way:** `let _ = ...` is Clippy's documented way to intentionally
silence the
[`unused_must_use`](https://rust-lang.github.io/rust-clippy/master/#unused_must_use)
lint at one specific, reviewed call site — see [`_`](../punctuation/underscore.md)
for why this differs from a bare statement, which doesn't reliably
suppress the warning at all.

## Embedded Rust Notes

**Full support.** `#[must_use]` is a pure compile-time diagnostic with no
runtime cost, so it applies identically in `#![no_std]` — `Result` is
`#[must_use]` in `core` exactly as it is in `std`, which matters
particularly in embedded code where an ignored peripheral-write error
(say, an I2C transaction that silently failed) can be far harder to debug
after the fact than on a hosted system with more visible failure modes.

---
title: "else"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: [Expression-oriented language, "if let / while let"]
related_syntax: [if, let]
see_also: [if]
---

## Explanation

`else` supplies the alternative branch to an `if` (or the diverging branch
of a `let ... else`). It cannot appear on its own — it is always the tail
of an `if` or `let` construct.

Chained `else if` is not special syntax — it's just an `if` expression
nested directly inside the `else` branch, formatted without extra braces
by convention.

In `let PATTERN = expr else { ... };` (let-else), the `else` block runs
only when the pattern fails to match, and that block is required to
diverge — it must `return`, `break`, `continue`, `panic!`, or otherwise
produce type `!` (e.g. `std::process::exit`), since control
flow cannot continue past it without the pattern's bindings having been
established.

## Usage examples

### Providing the alternative branch of an if

```
let x = 5;
if x > 0 {
    println!("positive");
} else {
    println!("non-positive"); // <- `else` runs when the `if` condition is false
}
```

### Branching on data (pattern matching)

A chain of `else if` branches testing the same value one case at a time
still compiles fine, but once every branch tests a different variant of
the same enum, that's the signal to reach for `match` instead — the
compiler then checks that every variant is handled, rather than trusting
the chain's final `else` to catch what's left.

```
enum ConnectionState { Connecting, Connected, Disconnected, Failed(String) }

// AVOID: an else-if chain re-testing the same value, one variant at a time
fn describe_avoid(state: &ConnectionState) -> &str {
    if matches!(state, ConnectionState::Connecting) {
        "connecting"
    } else if matches!(state, ConnectionState::Connected) { // <- each `else if` re-tests the same value against one more variant
        "connected"
    } else if matches!(state, ConnectionState::Disconnected) {
        "disconnected"
    } else { // <- silently also catches `Failed(_)`, discarding its message
        "failed"
    }
}

// PREFER: `match` forces every variant to be handled explicitly
fn describe(state: &ConnectionState) -> &str {
    match state {
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Failed(_) => "failed",
    }
}
```

A `match` on an enum is checked for exhaustiveness at
compile time, so adding a new variant later forces every `match` on it to
be updated — an `else`-if chain gives no such guarantee, per the
[Book's chapter on `match`](https://doc.rust-lang.org/book/ch06-02-match.html).
See [`if`](if.md) for the fuller `if`/`else` treatment.

## Embedded Rust Notes

**Full support.** No `std` dependency — behaves identically in `#![no_std]`.

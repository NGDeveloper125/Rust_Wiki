---
title: "if"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language, Pattern Matching]
related_syntax: [else, match]
see_also: [else, match]
---

## Explanation

`if` evaluates a boolean condition and branches accordingly. The condition
must be a `bool` — Rust has no implicit truthiness conversion from
integers, pointers, or `Option`, unlike C or Python. Parentheses around
the condition are not required (and are conventionally omitted); the
braces around each branch's body are mandatory, even for a single
statement.

Critically, `if` is an **expression**, not just a statement: an
`if`/`else` chain where every branch is a single expression (no trailing
semicolon) produces a value, and all branches must produce the same
type — for example, `if x > 0 { "positive" } else { "non-positive" }`
evaluates to a `&str`.

An `if` with no matching `else` always has type `()`, since the "value"
when the condition is false must exist and match the other branch's type.

`if let PATTERN = expr { ... }` is a distinct form — see the `let`-else
family under pattern matching — that matches a single pattern instead of
testing a `bool`.

## Basic usage example

```
let x = 5;
if x > 0 { // <- `if` branches on a boolean condition
    println!("positive");
}
```

## Best practices & deeper information

### Scenario: Validating input

Rejecting an out-of-range value at the top of a function, before doing
any real work, is the guard-clause shape — `if` tests the invalid case
and returns immediately, leaving the rest of the function to assume valid
input.

```
fn set_volume(level: i32) -> Result<(), String> {
    if !(0..=100).contains(&level) {
        // <- `if` as a guard clause: reject bad input before doing any real work
        return Err(format!("volume {level} out of range"));
    }
    // ... apply the validated level
    Ok(())
}
```

**Why this way:** checking the invalid case first and returning keeps the
rest of the function at one indentation level instead of nesting the
valid path inside an `if`, an idiom the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
collection favors for reducing nesting.

### Scenario: Handling and propagating errors

A malformed port string should stop `read_port` before any further
processing happens — `if` here tests one condition at a time and returns
early for each failure case.

```
fn read_port(raw: &str) -> Result<u16, String> {
    let Ok(port) = raw.trim().parse::<u16>() else {
        return Err(format!("invalid port: {raw:?}"));
    };
    if port == 0 {
        // <- ordinary `if` used for a second early-return guard mid-function
        return Err("port 0 is reserved".to_string());
    }
    Ok(port)
}
```

**Why this way:** `return` inside an `if` is type `!` (never), which
coerces to whatever type the surrounding context expects — the
[Reference's page on the never type](https://doc.rust-lang.org/reference/types/never.html)
is why this early exit can appear alongside other branches without a type
mismatch.

### Scenario: Branching on data (pattern matching)

Logging only the valid variant of a two-variant enum doesn't need a full
`match` — `if let` matches the one pattern that matters and silently does
nothing for the rest.

```
enum Reading { Valid(f64), Error(String) }

fn log_reading(reading: &Reading) {
    if let Reading::Valid(value) = reading {
        // <- `if let` matches one pattern without needing an exhaustive `match`
        println!("reading: {value}");
    }
}
```

**Why this way:** `if let` is exactly for the case where only one pattern
needs handling and the rest can be ignored — the
[Book's section on `if let`](https://doc.rust-lang.org/book/ch06-03-if-let.html)
recommends it over `match` precisely to avoid writing a wildcard `_` arm
that does nothing.

## Embedded Rust Notes

**Full support.** No dependency on `std`. Extremely common in embedded
code for polling a hardware flag/register bit before proceeding.

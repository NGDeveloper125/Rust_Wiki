---
title: "return"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: [Functions, Expression-oriented language]
related_syntax: [fn]
see_also: [fn]
---

## Explanation

`return` exits a function immediately with a given value.

Because Rust is expression-oriented, `return` is rarely required for the
*final* value of a function — the last expression in the body (with no
trailing semicolon) is returned implicitly. `return` exists for **early**
returns, typically from inside a conditional or loop, where control needs
to leave the function before reaching its end.

`return;` with no value is shorthand for `return ();` and is only valid
when the function's return type is `()`. `return` is itself an expression
of type `!` (never) — it never evaluates to anything at its own call
site, because control has already left — which lets it appear in
expression position, e.g. `let x = if cond { return; } else { 5 };`.

## Basic usage example

```
fn abs(x: i32) -> i32 {
    if x < 0 {
        return -x; // <- exits the function immediately with `-x`
    }
    x
}
```

**Convention:** `return` is rarely used for a function's final value —
the last expression in the body (no trailing `;`) is returned implicitly,
and idiomatic Rust reserves an explicit `return` for early exits like the
branch above. A trailing `return x;` is perfectly legal, just unidiomatic.

## Best practices & deeper information

### Scenario: Handling and propagating errors

Inside a single `match` arm that needs to diverge from the rest of the
function, `return` is often clearer than restructuring the whole function
around `?`.

```
fn parse_temperature(raw: &str) -> Result<f64, String> {
    let value = match raw.trim().parse::<f64>() {
        Ok(v) => v,
        Err(e) => return Err(format!("invalid temperature {raw:?}: {e}")), // <- exits the function early with the error
    };
    Ok(value)
}
```

**Why this way:** when the error needs to be reformatted rather than
passed straight through, `return` inside the failing arm reads more
directly than threading a `map_err` into a `?` chain — see the
[Book's chapter on recoverable errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html).

### Scenario: Validating input

A guard clause at the top of a function checks for invalid input and
`return`s immediately, so the rest of the function can assume the value
is valid.

```
fn set_fan_speed(percent: i32) -> Result<(), String> {
    if !(0..=100).contains(&percent) {
        return Err(format!("fan speed {percent}% out of range")); // <- guard clause: bail out before doing any real work
    }
    Ok(())
}
```

**Why this way:** checking the invalid case first and returning keeps the
rest of the function at a single indentation level instead of nesting the
valid path inside an `if` — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
favor this guard-clause shape over deep nesting.

## Embedded Rust Notes

**Full support.** No `std` dependency. Note that a `#![no_std]` binary's
`fn main() -> !` never returns at all — `return` is used for early exits
from ordinary functions, same as on a hosted target.

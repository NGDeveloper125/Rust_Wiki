---
title: "Result<T, E>"
area: "Error Handling"
embedded_support: full
groups: ["Error Handling", "Functional Programming", "Handling Errors & Failure", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Haskell / functional languages"]
related_syntax: [match, "if let", "?"]
see_also: ["Option<T>", "The ? operator (concept angle)", "Custom error types", "The Error trait"]
---

## Explanation

`Result<T, E>` is Rust's type for an operation that might not succeed,
and needs to say why when it doesn't. Like [`Option<T>`](option.md) it's
an ordinary two-variant enum — `Ok(T)` holding the success value, `Err(E)`
holding the failure — but where `Option` only records that something is
missing, `Result` carries a full value describing *what went wrong*, so a
caller can log it, match on its kind, or wrap it with more context.

It exists to replace exceptions. Languages built around `try`/`catch` let
a function throw a value whose type doesn't appear anywhere in its
signature, so nothing at compile time tells a caller which calls can fail
or what they can fail with. Rust makes failure part of the type: a
function that can fail returns `Result<T, E>`, so the compiler forces
every caller to at least acknowledge the possibility, whether by
matching, propagating with [`?`](the-question-mark-operator.md), or
explicitly choosing to `unwrap()` and accept a [panic](panic-and-unwinding.md).

The mental model is a fork in the road with data riding on both branches:
every `Result`-returning call produces exactly one of two outcomes, and
code downstream literally cannot reach the success value without going
through that fork. This is stricter than "hope the exception doesn't
happen" — there's no path through the code that silently skips handling
failure.

`Result` composes rather than being handled one match arm at a time.
Combinators like `.map()` (transform the `Ok` value), `.map_err()`
(transform the `Err` value, often to adapt one layer's error type into
another's — see [custom error types](custom-error-types.md)),
`.and_then()` (chain a further fallible step), and `.unwrap_or()` (supply
a fallback) let a pipeline of fallible steps read linearly instead of as
nested matches. `?` takes this composition further, turning an early
return into a single character at each fallible step.

What `E` should actually be is a design decision in its own right: a
quick script might get away with `Result<T, String>`, but a well-designed
library defines its own error enum (see
[custom error types](custom-error-types.md)) that implements the standard
[`Error` trait](the-error-trait.md), so callers get a stable, matchable,
composable failure type instead of an opaque message.

## Basic usage example

```
fn parse_reading(raw: &str) -> Result<f64, std::num::ParseFloatError> { // <- explicit "maybe failed" return type
    raw.trim().parse()
}

match parse_reading("21.5") {
    Ok(value) => println!("reading: {value}"),
    Err(e) => println!("invalid reading: {e}"),
}
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

A sensor driver's raw reading can fail to parse; the caller turns that
low-level error into a domain-specific one with `map_err` rather than
exposing `ParseFloatError` to its own callers.

```
#[derive(Debug)]
struct SensorError(String);

fn read_temperature(raw: &str) -> Result<f64, SensorError> {
    raw.trim()
        .parse::<f64>()
        .map_err(|e| SensorError(format!("bad reading {raw:?}: {e}"))) // <- Result::map_err adapts the error type
}

fn average(raws: &[&str]) -> Result<f64, SensorError> {
    let mut total = 0.0;
    for raw in raws {
        total += read_temperature(raw)?;
    }
    Ok(total / raws.len() as f64)
}
```

**Why this way:** converting a lower-level error into the caller's own
domain error at the boundary keeps failure information relevant to
callers instead of leaking implementation details, an approach the
[Rust Book's error handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
and the API Guidelines both recommend at library boundaries.

### Scenario: Validating input

Constructing an `Order` should be impossible with an invalid quantity, so
the constructor returns `Result<Order, OrderError>` instead of silently
clamping or panicking.

```
#[derive(Debug, PartialEq)]
enum OrderError {
    ZeroQuantity,
}

struct Order {
    sku: String,
    quantity: u32,
}

impl Order {
    fn new(sku: &str, quantity: u32) -> Result<Order, OrderError> { // <- fallible constructor: invalid orders can't exist
        if quantity == 0 {
            return Err(OrderError::ZeroQuantity);
        }
        Ok(Order { sku: sku.to_string(), quantity })
    }
}

let result = Order::new("SKU-1", 0);
assert!(matches!(result, Err(OrderError::ZeroQuantity))); // <- Result<Order, OrderError> makes failure explicit
```

**Why this way:** returning `Result` from the constructor makes "quantity
must be nonzero" a compile-time-checked part of the type's contract
instead of a comment, applying the parse-don't-validate idiom from
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) and
[Effective Rust](https://effective-rust.com/).

### Scenario: Testing

A unit test checks that invalid input produces the expected `Err`
variant, not just that the function "didn't crash."

```
fn divide_evenly(total: u32, parts: u32) -> Result<u32, String> {
    if parts == 0 {
        return Err("cannot divide by zero parts".to_string());
    }
    if total % parts != 0 {
        return Err(format!("{total} does not divide evenly into {parts} parts"));
    }
    Ok(total / parts)
}

#[test]
fn rejects_uneven_split() {
    let outcome = divide_evenly(10, 3); // <- Result lets the test assert on *why* it failed
    assert!(outcome.is_err());
    assert_eq!(outcome.unwrap_err(), "10 does not divide evenly into 3 parts");
}
```

**Why this way:** asserting on the specific `Err` payload, not just that
the call failed, catches regressions where the function still fails but
for the wrong reason or with a broken message — consistent with the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
on writing assertions that check specific outcomes.

## Embedded Rust Notes

**Full support.** `Result<T, E>` is `core::result::Result`, works
identically in `#![no_std]`, and requires no allocator by itself — only
the choice of `E` (say, a `String`-based error, which needs `alloc`)
can pull in a dependency the type itself doesn't require.

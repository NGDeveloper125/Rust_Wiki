---
title: "Functions"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures"]
related_syntax: [fn, "->", return, "( )"]
see_also: ["Expression-oriented language", "Closures & capturing", "Higher-order functions"]
---

## Explanation

A function is a named, reusable block of code: it takes zero or more typed
parameters, runs a body of statements and expressions, and produces a
value of a declared return type. Once written, a function can be called
from anywhere it's in scope, as many times as needed, without repeating
its body — the most basic unit of abstraction and code reuse in Rust, and
in almost every programming language before it.

Functions exist so that a piece of logic can be named, tested, and
changed in one place instead of copied wherever it's needed. In Rust
specifically, a function's signature is also a contract: every parameter
has an explicit type (never inferred from how it's called, unlike a
closure's parameters — see [Closures & capturing](closures-and-capturing.md)),
and the return type is stated up front, so both the compiler and the
reader know exactly what a function expects and promises without reading
its body.

A function's body follows Rust's [expression-oriented](expression-oriented-language.md)
rules: the final expression, written without a trailing semicolon, is the
value the function returns. `return` is only needed to exit early, from
somewhere other than the last line — this is why so many idiomatic Rust
functions have no explicit `return` statement at all.

Functions are also the foundation everything else in this group builds
on. A closure is essentially a function value that can additionally
capture variables from its surrounding scope; a
[higher-order function](higher-order-functions.md) is simply a function
that takes or returns another function (or closure); and a
[function pointer](function-pointers.md) is the plain, non-capturing type
a function's name itself has, distinct from a closure's type. Understanding
plain functions first is what makes each of those extensions make sense as
"a function, plus one specific capability," rather than unrelated new
syntax.

## Basic usage example

```
fn celsius_to_fahrenheit(celsius: f64) -> f64 { // <- a function: name, typed parameters, declared return type
    celsius * 9.0 / 5.0 + 32.0
}

let boiling = celsius_to_fahrenheit(100.0);
```

## Best practices & deeper information

### Scenario: Writing generic code

A function that clamps a value into a valid range is useful for any
orderable type — a sensor reading, a retry count, a percentage — so it's
written once, generic over `T`, instead of once per concrete type.

```
fn clamp_reading<T: PartialOrd>(value: T, min: T, max: T) -> T {
    // <- one function definition, generic over `T`, works for any orderable type
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

let capped_temp = clamp_reading(105.2, -40.0, 85.0);
let capped_retries = clamp_reading(12_i32, 0, 10);
```

**Why this way:** a generic function is monomorphized per concrete type
it's called with, so this costs nothing at runtime compared to hand-writing
`clamp_temp` and `clamp_retries` separately — the
[Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
covers the bound syntax used here.

### Scenario: Handling and propagating errors

Parsing a port number out of a configuration string can fail, so the
function that does the parsing declares `Result` as its return type
instead of panicking or returning a sentinel value like `0`.

```
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    // <- the function's signature makes failure part of its contract
    raw.trim().parse()
}

fn start_server(raw_port: &str) -> Result<(), std::num::ParseIntError> {
    let port = parse_port(raw_port)?;
    println!("listening on port {port}");
    Ok(())
}
```

**Why this way:** putting `Result` in the return type makes failure
visible to every caller at compile time instead of relying on
documentation or a runtime panic, the idiom the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
builds around.

### Scenario: Designing a public API

A function's parameter types are part of its API surface: accepting a
borrow instead of an owned value lets callers keep using their data
after the call, and returning an owned value keeps the function's
result independent of the caller's data.

```
pub struct Order {
    pub total_cents: u64,
}

pub fn total_with_tax(order: &Order, tax_rate: f64) -> u64 {
    // <- takes a borrow (caller keeps `order`), returns an owned u64
    (order.total_cents as f64 * (1.0 + tax_rate)).round() as u64
}
```

**Why this way:** favoring borrowed parameters over owned ones unless
ownership is genuinely needed gives callers the most flexibility, per the
[API Guidelines' flexibility checklist](https://rust-lang.github.io/api-guidelines/flexibility.html).

## Embedded Rust Notes

**Full support.** Functions are a core-language construct with zero
runtime cost beyond the call itself, and they compile identically whether
or not `std` is available. Free functions, methods, and associated
functions all work unchanged in `#![no_std]`; even interrupt handlers on
embedded targets are ordinary functions marked with a target-specific
attribute, not special syntax.

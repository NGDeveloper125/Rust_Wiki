---
title: "Function pointers (fn types)"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures"]
related_syntax: [fn, "->", "( )"]
see_also: ["Closures & capturing", "Fn / FnMut / FnOnce", "Higher-order functions"]
---

## Explanation

A function pointer is a plain value type like `fn(i32) -> i32`: it holds
only the address of a function, nothing else. Every named
[function](functions.md) has a concrete type of its own, but that type
can always be coerced to the matching `fn(...) -> ...` pointer type,
which is what you actually write down when a function's type needs to be
named — as a field type, a parameter type, or an array element type.

This is distinct from a [closure's](closures-and-capturing.md) type: a
closure that captures anything is backed by a compiler-generated
anonymous struct sized to hold exactly what it captured, so no single
named type describes every closure with a given signature. A function
pointer, in contrast, is always exactly one pointer wide, `Copy`, and
`'static`, regardless of which matching function it currently holds — the
tradeoff is that it can never carry an environment, since there's nowhere
in a bare pointer for captured state to live.

Because a function pointer captures nothing, it automatically implements
all three of [`Fn`, `FnMut`, and `FnOnce`](fn-fnmut-fnonce.md) — so any
place that accepts a closure by one of those bounds also accepts a plain
function name directly, with no extra syntax. This is also why a
[higher-order function](higher-order-functions.md) can choose to take a
bare `fn(...) -> ...` parameter instead of a generic closure bound when
it will never need to accept a capturing closure: doing so keeps the
function itself non-generic, at the cost of ruling out capturing closures
as arguments.

Reaching for a plain function pointer makes the most sense when every
value that will ever be stored is a non-capturing function anyway — a
dispatch table of named operations, a callback slot that's set once and
never needs surrounding context, or an FFI-style callback signature
shared with C.

## Basic usage example

```
fn square(n: i32) -> i32 { n * n }

let op: fn(i32) -> i32 = square; // <- `op`'s type is a function pointer, not a closure
println!("{}", op(6));
```

**Restriction:** only a function (or non-capturing closure) can be
assigned to a `fn(...) -> ...` binding — a closure that captures anything
fails to coerce, since there's no environment slot in a bare pointer to
hold it.

## Best practices & deeper information

### Scenario: Designing a public API

A logging sink's formatter never needs to capture request-specific
state, so its field is typed as a plain function pointer instead of a
boxed closure — smaller, `Copy`, and simpler to construct.

```
struct Logger {
    formatter: fn(level: &str, message: &str) -> String,
    // <- fn pointer: no capture ever needed, fixed-size, and Copy
}

fn plain_formatter(level: &str, message: &str) -> String {
    format!("[{level}] {message}")
}

impl Logger {
    fn new() -> Self {
        Logger { formatter: plain_formatter }
    }

    fn log(&self, level: &str, message: &str) {
        println!("{}", (self.formatter)(level, message));
    }
}
```

**Why this way:** a `Box<dyn Fn(...) -> ...>` field would also work here,
but it costs a heap allocation and a dynamic dispatch for a value that
never captures anything — the
[API Guidelines' flexibility guidance](https://rust-lang.github.io/api-guidelines/flexibility.html)
favors the simplest type that satisfies the actual requirement.

### Scenario: Writing generic code

A small calculator dispatches by operation name using a lookup table of
named operations, all sharing one concrete `fn(f64, f64) -> f64` type —
no generic parameter is needed since every entry has the same shape.

```
const OPERATIONS: &[(&str, fn(f64, f64) -> f64)] = &[
    // <- an array of fn pointers: every closure below captures nothing, so all coerce to one type
    ("add", |a, b| a + b),
    ("sub", |a, b| a - b),
    ("mul", |a, b| a * b),
];

fn run_operation(name: &str, a: f64, b: f64) -> Option<f64> {
    OPERATIONS
        .iter()
        .find(|(op_name, _)| *op_name == name)
        .map(|(_, op)| op(a, b))
}
```

**Why this way:** a non-capturing closure coerces to a function pointer
automatically wherever a `fn` type is expected, which the
[Rust Reference](https://doc.rust-lang.org/reference/type-coercions.html)
lists as one of the standard coercions — letting a table like this mix
closure literals and named functions under one concrete element type.

### Scenario: Working with collections

Applying one of several unit-conversion functions across a batch of
sensor readings is a single, non-generic helper: the conversion is a
parameter typed as a function pointer, not a generic closure bound.

```
fn to_fahrenheit(c: f64) -> f64 { c * 9.0 / 5.0 + 32.0 }
fn to_kelvin(c: f64) -> f64 { c + 273.15 }

fn convert_all(readings: &[f64], convert: fn(f64) -> f64) -> Vec<f64> {
    // <- `convert` is a plain fn pointer parameter, not a generic `F: Fn(f64) -> f64`
    readings.iter().map(|&c| convert(c)).collect()
}

let fahrenheit_readings = convert_all(&[0.0, 20.0, 100.0], to_fahrenheit);
```

**Why this way:** because a `fn` pointer parameter isn't generic,
`convert_all` compiles to a single function regardless of which
conversion function is passed in, unlike a generic `F: Fn(f64) -> f64`
parameter, which the
[standard library docs](https://doc.rust-lang.org/std/primitive.fn.html)
note gets monomorphized separately per distinct closure or function
passed in.

## Embedded Rust Notes

**Full support.** A function pointer is a plain address-sized value with
no dependency on an allocator, `std`, or an OS — it works identically in
`#![no_std]`, including as the concrete type behind a vendor HAL's
callback/interrupt-table entries, a common pattern in embedded C-interop
code carried straight over into Rust.

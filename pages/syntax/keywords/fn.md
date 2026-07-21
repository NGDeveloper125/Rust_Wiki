---
title: "fn"
kind: keyword
embedded_support: full
groups: [Basics, "Functions & Closures"]
related_concepts: [Functions, Closures & capturing, Higher-order functions]
related_syntax: ["->", "|"]
see_also: ["->"]
---

## Explanation

`fn` declares a function, giving it a name, a parameter list, and a body.

Every parameter must have an explicit type; unlike closures, `fn`
parameter and return types are never inferred from usage. Omitting the
`-> Type` return-type clause means the function returns `()` (unit). The
final expression in the body (no trailing semicolon) is the return value;
`return` is only needed for an early return.

`fn` also names a distinct family of **function pointer types** — a bare
`fn(i32, i32) -> i32` is a type you can hold in a variable, distinct from
the closure traits (`Fn`/`FnMut`/`FnOnce`). A function item defined with
`fn` can always be coerced to this function-pointer type as long as it
captures nothing.

`fn` can appear standalone (free functions), inside an `impl` block
(associated functions/methods, with or without a `self` receiver), inside
a `trait` (a method signature, optionally with a default body), and
nested inside another function body (an inner function — which, notably,
cannot capture variables from its enclosing scope; only closures can).

## Usage examples

### Declaring a function with parameters and a return type

```
fn add(a: i32, b: i32) -> i32 { a + b } // <- `fn` declares a function named `add`
```

### Writing generic code

A function that finds the largest element of a slice works for any
orderable, copyable type — `fn` declares it once, generic over `T`, and
the compiler generates a specialized version per concrete type used.

```
fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
    // <- `fn` here is generic over `T`, constrained by the bounds after the colon
    let mut max = items[0];
    for &item in items {
        if item > max {
            max = item;
        }
    }
    max
}

let highest_temp = largest(&[21.5, 19.8, 23.1]);
```

A generic `fn` is monomorphized once per concrete type
it's instantiated with (calls sharing a type `T` share one copy), so this
costs nothing at runtime compared to writing a separate `largest_f64`,
`largest_i32`, etc. — the
[Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
covers the bound syntax used here.

### Handling and propagating errors

Parsing a configuration value can fail, so the `fn` that does the parsing
declares its return type as `Result` rather than panicking or returning a
sentinel value.

```
fn parse_config(raw: &str) -> Result<u16, std::num::ParseIntError> {
    // <- `fn` declares the return type as `Result`, making failure part of the signature
    let port: u16 = raw.trim().parse()?;
    Ok(port)
}
```

Putting `Result` in the signature makes failure visible
to every caller at compile time instead of relying on documentation or a
panic at runtime, the idiom the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
builds around.

### Designing a public API

A public type's constructor is conventionally an inherent `fn` named
`new`, not a free function or a public field initializer.

```
pub struct Client {}

impl Client {
    pub fn new(host: &str) -> Self { // <- `fn` here follows the `new` constructor naming convention
        let _ = host;
        Client {}
    }

    pub fn send(&self, payload: &[u8]) -> Result<(), std::io::Error> {
        let _ = payload;
        Ok(())
    }
}
```

The
[API Guidelines' C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
item specifies constructors should be static, inherent `fn`s named `new`
whenever a type has an obvious default construction path.

## Explanation (Embedded)

`fn` means exactly the same thing under `#![no_std]` as in hosted Rust —
the same parameter/return-type rules, the same monomorphization of
generic functions, the same function-pointer coercion for a capturing-
nothing function item. The one place embedded code leans on `fn` in a way
rarely seen in hosted code is an interrupt handler: a plain `fn`, usually
with no parameters and no return value, wired into the vector table not
through special syntax or a different calling convention but through an
attribute a HAL/PAC crate provides (`#[interrupt]`, `#[exception]`) sitting
directly above an ordinary function declaration.

## Usage examples (Embedded)

### Declaring a HAL driver method signature

```
fn read_temperature(i2c: &mut impl embedded_hal::i2c::I2c) -> Result<f32, embedded_hal::i2c::ErrorKind> {
    // <- `fn` declares a driver function, generic over any I2C bus implementation
    let mut buf = [0u8; 2];
    i2c.write_read(0x48, &[0x00], &mut buf)?;
    Ok(i16::from_be_bytes(buf) as f32 / 256.0)
}
```

### An interrupt handler as an ordinary `fn`

```
#[interrupt]
fn TIM2() { // <- `fn` here: no parameters, no return value, wired to the vector table by the attribute above it
    // clear the timer's interrupt flag and handle the tick
}
```

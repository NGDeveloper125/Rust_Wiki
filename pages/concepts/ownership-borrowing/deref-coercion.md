---
title: "Deref & DerefMut coercion"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["*"]
see_also: ["Smart pointers (Box<T>)", "Shared ownership (Rc & Arc)"]
---

## Explanation

`Deref` and `DerefMut` let a smart pointer type transparently behave like
a reference to whatever it wraps. A type implementing `Deref<Target = T>`
can be used almost anywhere a `&T` is expected — most visibly, calling a
method defined on `T` directly on the smart pointer (`my_box.method()`
instead of `(*my_box).method()`), because the compiler automatically
inserts as many derefs as needed to find a matching method.

This coercion is what makes `Box<T>`, `Rc<T>`, `String` (whose
`Deref::Target` is `str`, so `&String` coerces to `&str`), and `Vec<T>`
(whose target is `[T]`, so `&Vec<T>` coerces to `&[T]`) feel ergonomic to use day
to day — you rarely need to think about the wrapper layer at all, because
method calls, and reference coercions in function-argument position, see
straight through it to the underlying type.

The tradeoff worth knowing about: overusing custom `Deref` impls purely
to fake inheritance-like "is-a" relationships between unrelated types is
a recognized anti-pattern in Rust (sometimes called "Deref polymorphism")
— `Deref` is meant to model "acts like a reference to," not "is a kind
of," and stretching it to the latter tends to produce confusing method
resolution rather than genuinely reusable abstraction.

## Basic usage example

```
fn greet(name: &str) {
    println!("hello, {name}");
}

let boxed = Box::new(String::from("world"));
greet(&boxed); // <- &Box<String> coerces through Box then String to &str
```

## Best practices & deeper information

### Scenario: Designing a public API

A function that only needs to read a string should accept `&str`, not
`&String` — deref coercion means callers holding either a `String` or a
string literal can call it without extra ceremony.

```
fn greet(name: &str) { // <- PREFER: &str accepts both &String (via coercion) and &str directly
    println!("hello, {name}");
}

// fn greet_narrow(name: &String) { ... } // AVOID: forces every caller to already own a String

let owned = String::from("Ada");
greet(&owned); // <- &String coerces to &str automatically
greet("Grace"); // a string literal is already &str; no coercion needed
```

**Why this way:** `&str` accepts everything `&String` does (deref
coercion covers that direction for free) plus string literals and other
`&str`-producing sources, so it's strictly the more widely callable
parameter type — the
[Rust Book](https://doc.rust-lang.org/book/ch04-03-slices.html#string-slices-as-parameters)
recommends `&str` over `&String` as the more general parameter type for
exactly this reason.

### Scenario: Writing generic code

A generic function bounded by `AsRef<str>` leans on the same "many
wrapper types act like a reference to the same target" idea deref
coercion embodies, letting one function body serve owned strings,
borrowed slices, and boxed strings alike.

```
fn shout<S: AsRef<str>>(value: S) -> String {
    value.as_ref().to_uppercase() // <- works whether `value` is String, &str, or Box<str>
}

let boxed: Box<str> = "quiet".into();
println!("{}", shout(&*boxed)); // <- Box<str> derefs to str, coercing to &str at the call boundary
println!("{}", shout(String::from("also quiet")));
```

**Why this way:** writing generic code against `AsRef<str>` lets one
function serve owned strings, borrowed slices, and smart-pointer-wrapped
strings without the caller manually unwrapping anything — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generics-c-generic)
recommend this for functions that only need read access to string-like
data.

## Explanation (Embedded)

`Deref`/`DerefMut` coercion is identical under `#![no_std]` — both traits
live in `core::ops`, and the compiler inserts the same automatic derefs
whether the target being deref'd to is `str`, `[T]`, or something else
entirely. The mechanism shows up constantly in idiomatic `no_std` code
because `heapless`'s fixed-capacity collections lean on it directly:
`heapless::Vec<T, N>` implements `Deref<Target = [T]>` the same way
`std::Vec<T>` does, so a `heapless::Vec` can be passed anywhere a plain
slice is expected, with no manual `.as_slice()` call needed. Driver crates
lean on it deliberately too: a thin newtype wrapper around a HAL
peripheral (`struct Debounced<P>(P)`) can implement `Deref<Target = P>` so
every method the wrapped peripheral exposes stays callable directly on the
wrapper — the wrapper adds behavior (debouncing, retries, a safety check)
without hand-writing a forwarding method for each one. As on a hosted
target, this is a distinct, legitimate use, separate from the "Deref
polymorphism" anti-pattern of stretching `Deref` to fake an inheritance
relationship between unrelated types — a debounce wrapper genuinely does
"act like" the peripheral it wraps, which is exactly what `Deref` is for.

## Basic usage example (Embedded)

```
fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, b| acc ^ b)
}

let buf: heapless::Vec<u8, 16> = heapless::Vec::from_slice(&[1, 2, 3]).unwrap();
checksum(&buf); // <- &heapless::Vec<u8, 16> derefs to &[u8]
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A function that only needs to read bytes should accept `&[u8]`, not a
specific collection type — deref coercion means callers holding a
`heapless::Vec` can call it with no extra ceremony.

```
fn checksum(data: &[u8]) -> u8 { // <- PREFER: &[u8] accepts arrays, slices, and heapless::Vec alike
    data.iter().fold(0, |acc, b| acc ^ b)
}

// fn checksum_narrow(data: &heapless::Vec<u8, 16>) -> u8 { ... } // AVOID: locks out arrays, other capacities, plain slices

let frame: heapless::Vec<u8, 16> = heapless::Vec::from_slice(&[0xAA, 0x01, 0x10]).unwrap();
checksum(&frame); // <- &heapless::Vec<u8, 16> coerces to &[u8]
checksum(&[0xAA, 0x01, 0x10]); // a plain array reference works too, no coercion needed
```

**Why this way:** `&[u8]` accepts everything a specific collection type
does (deref coercion covers `heapless::Vec` for free) plus arrays and
slices from any other source, making it the strictly more widely callable
parameter type — the same reasoning the
[Rust Book](https://doc.rust-lang.org/book/ch04-03-slices.html#string-slices-as-parameters)
gives for preferring `&str` over `&String` applies directly to preferring
`&[u8]` over a concrete `no_std` collection type.

### Scenario: Writing generic code

A thin wrapper around an embedded-hal I2C bus adds a retry policy without
needing to manually re-expose every method the underlying bus type has.

```
use core::ops::{Deref, DerefMut};

struct RetryingI2c<I2C>(I2C); // <- wraps a bus, adds retry behavior on top

impl<I2C> Deref for RetryingI2c<I2C> {
    type Target = I2C;
    fn deref(&self) -> &I2C { &self.0 }
}

impl<I2C> DerefMut for RetryingI2c<I2C> {
    fn deref_mut(&mut self) -> &mut I2C { &mut self.0 }
}

fn read_register(bus: &mut RetryingI2c<impl embedded_hal::i2c::I2c>, addr: u8) {
    bus.write(addr, &[0x00]).ok(); // <- calls I2c::write directly through the wrapper, no manual forwarding needed
}
```

**Why this way:** implementing `Deref`/`DerefMut` for a newtype wrapper is
the standard way to add behavior around a peripheral type without
hand-writing a forwarding method for every one the wrapped bus exposes —
this is `Deref` modeling "acts like a reference to the wrapped bus," the
legitimate use the classic explanation distinguishes from "Deref
polymorphism" fake inheritance.

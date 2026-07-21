---
title: "Default trait methods"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns"]
related_syntax: [trait]
see_also: ["Traits", "Supertraits"]
---

## Explanation

A trait method can carry a default body, used by any implementer that
doesn't override it — for example, a `Greet` trait's `greet` method could
have a default body that calls `self.name()` internally.

Here every implementer of `Greet` must define `name`, but gets `greet`
for free unless it chooses to override it. This lets a trait provide
substantial shared behavior — not just a bare contract — while still
letting each implementer customize any specific piece of it, similar to
how an abstract base class in an OO language can implement some methods
concretely while leaving others abstract, but without requiring an actual
inheritance relationship between the types involved.

## Basic usage example

```
trait Greet {
    fn name(&self) -> String;
    fn greet(&self) -> String { // <- default body, used unless the implementer overrides it
        format!("Hello, {}!", self.name())
    }
}

struct Cat;
impl Greet for Cat {
    fn name(&self) -> String { "Cat".into() } // greet() is inherited, not redefined
}

println!("{}", Cat.greet());
```

## Best practices & deeper information

### Scenario: Implementing traits

A trait with a default method lets most implementers rely on the shared
behavior while still allowing a specific type to override it when it can
do better.

```
trait Greet {
    fn name(&self) -> String;
    fn greet(&self) -> String { // <- default: used unless the implementer overrides it
        format!("Hello, {}!", self.name())
    }
}

struct Robot;
impl Greet for Robot {
    fn name(&self) -> String { "Unit-7".into() }
    fn greet(&self) -> String { // <- override: Robot needs different phrasing
        format!("GREETINGS. I AM {}.", self.name().to_uppercase())
    }
}
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html) covers
this as "using default implementations" — a default is chosen
per-implementer, not all-or-nothing, so most types can skip writing
`greet` entirely while the ones with real reasons to differ still can.

### Scenario: Designing a public API

Adding a default body to a trait method that previously had none lets a
library grow its trait's surface without forcing every downstream
implementer to add a new method just to keep compiling.

```
pub trait Plugin {
    fn name(&self) -> &str;
    fn on_shutdown(&self) {} // <- added later, with a default: existing implementers don't break
}

struct Logger; // written before on_shutdown existed
impl Plugin for Logger { // <- still compiles: on_shutdown's default covers it
    fn name(&self) -> &str { "logger" }
}
```

**Why this way:** a required method added to a public trait is a breaking
change for every downstream implementer; a method added with a default
body is usually *not* — [RFC 1105](https://rust-lang.github.io/rfcs/1105-api-evolution.html)
classifies it as a minor change (though a name collision with an inherent
method or another trait in scope can still cause downstream ambiguity
errors). Designing traits so behavior can be extended this way is a core
forward-compatibility technique.

## Explanation (Embedded)

Default methods work identically under `#![no_std]` — nothing about the
mechanism depends on `std` or an allocator. They're genuinely common in
`embedded-hal`-style traits specifically because a peripheral's real
hardware contract is usually one small, irreducible primitive, with every
higher-level operation expressible in terms of it: a serial/UART trait
only strictly needs a way to write a single byte, so it can declare
`write_byte` as the required method and provide `write_all` (looping over
a byte slice) as a default built entirely on top of it. Every vendor's
UART implementation then only has to supply the one primitive operation
its register interface actually performs, and gets the convenience method
for free — the same "required primitive, default convenience" shape the
[trait](../../syntax/keywords/trait.md) page describes for `embedded-hal`
generally, applied here to one trait's own internal method design rather
than to the trait-as-abstraction-boundary story.

## Basic usage example (Embedded)

```
trait SerialWrite {
    fn write_byte(&mut self, byte: u8); // required: the one primitive every UART must supply
    fn write_all(&mut self, bytes: &[u8]) { // <- default: built from write_byte alone
        for &b in bytes {
            self.write_byte(b);
        }
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

A UART trait shared across vendors should require only the one operation
that genuinely differs per chip — writing a single byte to a data
register — and provide everything built on top of it as a default.

```
trait SerialWrite {
    fn write_byte(&mut self, byte: u8); // <- required: the only per-chip-specific primitive
    fn write_all(&mut self, bytes: &[u8]) { // <- default: same for every implementer
        for &b in bytes {
            self.write_byte(b);
        }
    }
}

struct Usart1;
impl SerialWrite for Usart1 {
    fn write_byte(&mut self, byte: u8) {
        // a real impl would poll a status register, then write a data register
        let _ = byte;
    }
}

Usart1.write_all(b"ready\r\n"); // <- write_all comes for free once write_byte exists
```

**Why this way:** narrowing the required method to the one operation that
actually differs per peripheral minimizes what a vendor's HAL has to
implement by hand, while every consumer still gets `write_all` — the same
default-method shape the
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html) recommends
generally, particularly valuable in embedded code where hand-implementing
every convenience method per chip would otherwise multiply boilerplate
across every vendor's HAL crate.

### Scenario: Designing a public API

An `embedded-hal`-style trait can grow a new convenience method with a
default body without breaking every existing implementer already shipped
in a vendor's HAL crate.

```
pub trait SerialWrite {
    fn write_byte(&mut self, byte: u8);
    fn write_all(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_byte(b);
        }
    }
    fn flush(&mut self) {} // <- added later, with a no-op default: existing implementers keep compiling
}

struct Usart1; // written before flush() existed
impl SerialWrite for Usart1 { // <- still compiles: flush()'s default covers it
    fn write_byte(&mut self, byte: u8) {
        let _ = byte;
    }
}
```

**Why this way:** a HAL trait implemented by many independent vendor
crates can't ask every one of them to add a method the moment the trait
grows — a default body keeps the addition non-breaking, the same
forward-compatibility reasoning [RFC
1105](https://rust-lang.github.io/rfcs/1105-api-evolution.html) applies
to trait evolution generally, with the stakes raised in an ecosystem where
`SerialWrite` might have implementers in a dozen different HAL crates the
trait's own author doesn't control.

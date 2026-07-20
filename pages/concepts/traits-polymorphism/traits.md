---
title: "Traits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns", "Decoupling", "Coming from Haskell / functional languages"]
related_syntax: [trait, impl]
see_also: ["Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)", "Default trait methods"]
---

## Explanation

A trait defines a set of behavior — methods a type promises to provide —
that any type can then implement, independent of that type's own
definition. It's Rust's answer to "shared behavior across unrelated
types," filling the role interfaces play in Java/C#, protocols in Swift,
or type classes in Haskell.

```
struct Cat;

trait Greet {
    fn greet(&self) -> String;
}
impl Greet for Cat {
    fn greet(&self) -> String { "meow".into() }
}
```

Because traits are implemented separately from a type's definition (via
`impl Trait for Type`, potentially in a completely different module or
even a different crate than the type itself), a type can implement any
number of unrelated traits without any of them needing to know about each
other — this is the mechanism behind
[decoupling](dependency-injection-via-traits.md) code from
concrete types: a function can depend on "anything implementing this
trait" instead of a specific concrete type, without the trait author and
the type author ever needing to coordinate directly.

Traits are also how Rust achieves polymorphism without classical
inheritance — see
[trait objects & dynamic dispatch](trait-objects-dynamic-dispatch.md) for
the runtime-polymorphism side, and
[static dispatch & monomorphization](static-dispatch-monomorphization.md)
for the compile-time side.

## Basic usage example

```
trait Greet {
    fn greet(&self) -> String;
}

struct Cat;
impl Greet for Cat { // <- Cat implements the Greet trait
    fn greet(&self) -> String { "meow".into() }
}

println!("{}", Cat.greet());
```

## Best practices & deeper information

### Scenario: Implementing traits

A domain type gains broad interoperability by implementing a standard
trait like `Display`, rather than a bespoke method — any code already
written against `Display` accepts it for free.

```
struct Temperature { celsius: f64 }

impl std::fmt::Display for Temperature { // <- implementing a trait, not a one-off to_string() method
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

fn log(value: &impl std::fmt::Display) { // <- accepts anything implementing Display
    println!("reading: {value}");
}

log(&Temperature { celsius: 21.5 });
```

**Why this way:** eagerly implementing common standard traits lets a type
slot directly into generic code, formatting macros, and collections that
already know how to work with them, instead of every caller needing a
type-specific method — the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html)
codifies this.

### Scenario: Designing a public API

Shipping a trait as a library's extension point, instead of a concrete
struct, is a deliberate design choice — callers implement the trait for
their own types instead of being locked into the crate's.

```
pub trait Storage { // <- the crate's extension point is a trait, not a fixed struct
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

pub struct Cache<S: Storage> {
    backend: S,
}

impl<S: Storage> Cache<S> {
    pub fn refresh(&mut self, key: &str, value: String) {
        self.backend.set(key, value);
    }
}
```

**Why this way:** keeping the trait narrow (two methods, nothing about how
storage is implemented) leaves room to add backends later without
breaking existing implementers — see the
[API Guidelines on future-proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
for the broader case around designing traits as stable extension points.

### Scenario: Testing

Defining a trait around a dependency — even one with a single real
implementation — opens the door to a test-only stand-in, without any
conditional compilation inside the production code path.

```
trait Clock {
    fn now(&self) -> u64;
}

struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> u64 { /* reads the real system time */ 1_700_000_000 }
}

struct FixedClock(u64); // <- test-only implementation of the same trait
impl Clock for FixedClock {
    fn now(&self) -> u64 { self.0 }
}

fn is_expired(clock: &impl Clock, expiry: u64) -> bool {
    clock.now() > expiry
}

assert!(is_expired(&FixedClock(2_000_000_000), 1_000_000_000)); // no real clock involved
```

**Why this way:** depending on the trait rather than `SystemClock` directly
means the test controls time deterministically instead of racing the real
clock — a standard use of trait seams for testability, as the
[Rust Book's mock-object example](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#a-use-case-for-interior-mutability-mock-objects)
illustrates with the same trait-double approach.

## Embedded Rust Notes

**Full support.** Traits are core-language and allocator-free. The
`embedded-hal` crate is the clearest real-world proof of how central this
is to embedded Rust: an entire ecosystem of hardware-agnostic driver
crates is built purely on trait abstractions, letting one driver crate
work across dozens of unrelated vendors' microcontrollers.

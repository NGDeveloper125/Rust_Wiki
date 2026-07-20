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
doesn't override it:

```
trait Greet {
    fn name(&self) -> String;
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

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

## Embedded Rust Notes

**Full support.** No allocator dependency — default methods work
identically in `#![no_std]`.

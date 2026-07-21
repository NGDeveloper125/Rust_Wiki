---
title: "trait"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism"]
related_concepts: [Traits, "Default trait methods", Supertraits, "Associated types"]
related_syntax: [impl, where, ":", "+"]
see_also: [impl, dyn]
---

## Explanation

`trait` declares a named set of behavior: method signatures a type can
promise to provide, as in `trait Greet { fn greet(&self) -> String; }`.
A trait by itself defines no data and no memory layout — it's a contract,
implemented for a concrete type separately via [`impl`](impl.md).

Inside the braces, a `trait` body can hold:

- **Method signatures with no body** (`fn greet(&self) -> String;`) —
  required, every implementer must supply one.
- **Method signatures with a default body** (`fn greet(&self) -> String { ... }`) —
  optional to override; see [Default trait methods](../../concepts/traits-polymorphism/default-trait-methods.md).
- **Associated types** (`type Item;`) — a placeholder type each
  implementer fixes to something concrete, as `Iterator` does with
  `type Item;`.
- **Associated consts** (`const MAX: u32;`) — a named constant each
  implementer supplies, resolved per-implementation rather than shared.

A trait can also declare **supertraits** with a `:` after its name —
`trait Sub: Super` requires any `Sub` implementer to also implement
`Super`, guaranteeing `Super`'s methods are available inside `Sub`'s own
default bodies. See [Supertraits](../../concepts/traits-polymorphism/supertraits.md)
for why this exists and what it buys you; the `:` here is the same trait
bound syntax used on generic parameters, just attached to a trait
declaration instead of a type parameter.

## Usage examples

### Declaring a trait's method contract

```
trait Greet { // <- `trait` declares the contract, no implementation yet
    fn greet(&self) -> String;
}
```

### Implementing traits

A trait that mixes one required method with one default method lets every
implementer supply only the irreducible part, while sharing the rest.

```
trait Greet {
    fn name(&self) -> String; // <- required: no body, every implementer must define it
    fn greet(&self) -> String { // <- optional: default body, inherited unless overridden
        format!("Hello, {}!", self.name())
    }
}

struct Visitor;
impl Greet for Visitor {
    fn name(&self) -> String { "Visitor".into() } // greet() comes for free
}

println!("{}", Visitor.greet());
```

Declaring `name` as required and `greet` as a default
lets the `trait` capture exactly one piece of per-type information while
generating the rest, which the
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html) covers as
the standard shape for a trait that's more than a bare method list.

### Designing a public API

A crate that wants callers to plug in their own storage backend declares
a narrow `trait` as the extension point, rather than exposing an internal
struct for callers to depend on directly.

```
pub trait Storage { // <- `trait` is the extension point; callers implement it for their own types
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: String);
}

pub struct KeyValueStore<S: Storage> {
    backend: S,
}

impl<S: Storage> KeyValueStore<S> {
    pub fn refresh(&mut self, key: &str, value: String) {
        self.backend.set(key, value);
    }
}
```

Keeping the `trait`'s method list minimal leaves room to
add backends later without breaking existing implementers — see
[Traits](../../concepts/traits-polymorphism/traits.md) for the fuller
design rationale behind shipping a trait as an API boundary.

## Embedded Rust Notes

**Full support.** `trait` declarations are core-language and cost nothing
at runtime — no allocator or `std` dependency. `embedded-hal`'s entire
hardware-abstraction model is built from `trait` declarations implemented
by vendor-specific driver crates.

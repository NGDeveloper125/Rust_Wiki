---
title: "The newtype pattern"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models", "Unique to Rust"]
related_syntax: [struct]
see_also: ["Tuple structs", "Type aliases", "The orphan rule & coherence"]
---

## Explanation

The newtype pattern wraps an existing type in a single-field tuple struct
to give it a distinct identity:

```
struct Meters(f64);
struct Seconds(f64);
```

Even though both wrap `f64`, `Meters` and `Seconds` are different types
to the compiler — passing a `Seconds` value where `Meters` is expected is
a compile error, catching a whole category of unit-confusion bugs
(the kind that has, famously, destroyed real spacecraft) that a plain
type alias (which is fully interchangeable with what it aliases) would
not catch at all.

Newtypes serve a second, unrelated purpose too: Rust's
[orphan rule](../traits-polymorphism/the-orphan-rule-and-coherence.md)
forbids implementing a foreign trait on a foreign type directly (e.g. you
can't `impl Display for Vec<T>` from outside the crate that defines
either), but wrapping the foreign type in your own newtype gives you a
type you *do* own, on which you're free to implement whatever traits you
like. This combination — meaningful type safety plus a way around the
orphan rule — is common enough that it's considered one of Rust's
signature idioms rather than a niche trick.

## Basic usage example

```
struct Meters(f64); // <- a distinct type, not just another name for f64

fn print_distance(d: Meters) {
    println!("{} m", d.0); // <- the wrapped value is accessed via .0
}

print_distance(Meters(5.0));
// print_distance(5.0); // <- would not compile: f64 is not a Meters
```

**Restriction:** the wrapper gets no behavior for free — arithmetic
operators, trait impls, and methods that `f64` has don't automatically
apply to `Meters`; each one needed on the newtype has to be implemented
explicitly (e.g. `impl Add for Meters`).

## Best practices & deeper information

### Scenario: Designing a public API

Two parameters of the same primitive type, back to back, are exactly the
shape that invites an accidental argument swap that compiles cleanly and
misbehaves at runtime. Wrapping each in its own newtype turns that swap
into a compile error.

```
struct UserId(u64);  // <- distinct from OrderId even though both wrap u64
struct OrderId(u64);

fn refund(user: UserId, order: OrderId) { // <- swapping the argument order is now a compile error
    println!("refunding order {} for user {}", order.0, user.0);
}

refund(UserId(42), OrderId(1001));
// refund(OrderId(1001), UserId(42)); // <- would not compile: types don't match
```

**Why this way:** [Effective Rust](https://effective-rust.com/) covers
newtypes as a primary defense against this exact class of bug — two
`u64` parameters are indistinguishable to the compiler, but `UserId` and
`OrderId` are not, so a mixed-up call site is caught before it ships.

### Scenario: Converting between types

Implementing `From`/`Into` in both directions gives a newtype
conventional wrapping and unwrapping, instead of an ad-hoc method name
every newtype in the codebase would otherwise invent independently.

```
struct UserId(u64);

impl From<u64> for UserId {
    fn from(raw: u64) -> Self { // <- wrapping goes through From, not a custom method name
        UserId(raw)
    }
}

impl From<UserId> for u64 {
    fn from(id: UserId) -> Self { // <- unwrapping is just as conventional as wrapping
        id.0
    }
}

let id: UserId = 42.into();  // <- wrap
let raw: u64 = id.into();    // <- unwrap
```

**Why this way:** the API Guidelines'
[C-CONV](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv)
recommend the standard `From`/`Into` traits over one-off conversion
methods — doing so lets the newtype interoperate with any API already
written against `Into`, for free.

## Embedded Rust Notes

**Full support.** No allocator dependency — this is one of the most-used
patterns in embedded HAL crates specifically, wrapping raw register
values and pin numbers in distinct types to prevent unit-confusion bugs
(millivolts vs. raw ADC counts, for example).

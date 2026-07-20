---
title: "Supertraits"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code"]
related_syntax: [trait, ":"]
see_also: ["Traits", "Trait bounds"]
---

## Explanation

A trait can require that any implementer also implement another trait —
its supertrait:

```
trait Named {
    fn name(&self) -> String;
}
trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

Here `Greet: Named` means "you can only implement `Greet` if you've also
implemented `Named`" — which is what lets `Greet`'s default method call
`self.name()` at all, since that method is guaranteed to exist on any
type this trait is implemented for. This is Rust's closest equivalent to
interface inheritance, but it composes rather than creates a hierarchy:
a trait can have several supertraits, and unrelated traits can share the
same supertrait without any of them being related to each other beyond
that one shared requirement.

## Basic usage example

```
trait Named {
    fn name(&self) -> String;
}

trait Greet: Named { // <- Greet requires Named too
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat;
impl Named for Cat { fn name(&self) -> String { "Cat".into() } }
impl Greet for Cat {} // only allowed because Cat also implements Named

println!("{}", Cat.greet());
```

## Best practices & deeper information

### Scenario: Implementing traits

The standard library itself leans on supertraits: `Eq` requires
`PartialEq`, so implementing `Eq` for a type is only possible once
`PartialEq` is implemented too.

```
#[derive(PartialEq)] // <- required first: Eq's supertrait
struct SensorId(u32);

impl Eq for SensorId {} // only allowed because SensorId already implements PartialEq

fn has_target<T: Eq>(ids: &[T], target: &T) -> bool { // <- Eq usable here because SensorId satisfies it
    ids.contains(target)
}

has_target(&[SensorId(1), SensorId(2)], &SensorId(2));
```

**Why this way:** `Eq` adds no methods of its own — it only asserts that
`PartialEq`'s equality is total (reflexive for every value) — which is
exactly the kind of "implementer promises an extra property" role a
supertrait exists to express; see
[`std::cmp::Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html).

### Scenario: Writing generic code

A generic function bounded by a trait that has a supertrait can call the
supertrait's methods too, without adding a second bound — the supertrait
relationship already guarantees it's implemented.

```
trait Named {
    fn name(&self) -> String;
}
trait Reportable: Named { // <- Reportable's supertrait guarantees name() exists
    fn severity(&self) -> u8;
}

fn report<T: Reportable>(item: &T) -> String {
    format!("[{}] severity {}", item.name(), item.severity()) // <- name() usable with only a Reportable bound
}

struct DiskAlert;
impl Named for DiskAlert { fn name(&self) -> String { "disk".into() } }
impl Reportable for DiskAlert { fn severity(&self) -> u8 { 3 } }

report(&DiskAlert);
```

**Why this way:** writing `fn report<T: Reportable + Named>` would be
redundant — a supertrait bound is elaborated, so `T: Reportable` already
implies `T: Named` wherever it's required, as the
[Rust Reference's section on supertraits](https://doc.rust-lang.org/reference/items/traits.html#supertraits)
describes.

## Embedded Rust Notes

**Full support.** No `std`/allocator dependency.

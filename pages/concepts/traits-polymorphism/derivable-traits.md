---
title: "Derivable traits (Debug, Clone, PartialEq, …)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Serialization"]
related_syntax: [derive]
see_also: ["Traits", "Copy vs Clone"]
---

## Explanation

Many common traits have an obvious, mechanical implementation that can be
generated automatically from a type's structure — comparing a struct
field-by-field for `PartialEq`, printing every field for `Debug`, cloning
every field for `Clone`. `#[derive(...)]` generates exactly that
mechanical implementation, saving the boilerplate of writing it by hand:

```
#[derive(Debug, Clone, PartialEq)]
struct Point { x: f64, y: f64 }
```

A derive only works if every field's own type already implements the
trait being derived — deriving `Clone` for a struct containing a
non-`Clone` field is a compile error, since there'd be no way to generate
a working implementation. This is also the mechanism behind
`serde`'s `#[derive(Serialize, Deserialize)]` — the same
generate-from-structure idea, extended by a third-party procedural macro
rather than the small fixed set of derives built into the compiler
itself, which is why the [serialization](../../concepts/serialization.md)
ecosystem is able to plug into ordinary struct/enum definitions with a
single attribute rather than requiring hand-written conversion code.

## Basic usage example

```
#[derive(Debug, Clone, PartialEq)] // <- mechanically generates all three impls
struct Point { x: f64, y: f64 }

let a = Point { x: 1.0, y: 2.0 };
let b = a.clone();
println!("{:?} {}", b, a == b);
```

**Restriction:** a derive only works if every field's own type already
implements the trait being derived — a struct with a non-`Clone` field
cannot itself derive `Clone`.

## Best practices & deeper information

### Scenario: Implementing traits

`#[derive(PartialEq)]` covers a plain field-by-field comparison, but a
type with a field that shouldn't count toward equality — a cache, a
timestamp — needs a manual `impl` instead.

```
struct Reading {
    value: f64,
    measured_at: u64, // shouldn't affect equality
}

impl PartialEq for Reading { // <- manual: derive would also compare measured_at
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
```

**Why this way:** `#[derive(...)]` always compares/prints/clones *every*
field — the moment equality (or `Debug`, or `Clone`) needs to mean
something other than "all fields, mechanically," a manual `impl` is the
only option; the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html)
still recommends implementing the trait, just not necessarily by
deriving it.

### Scenario: Testing

`assert_eq!` requires both `PartialEq` (to compare) and `Debug` (to print
the values in the failure message) — deriving both is what makes ordinary
equality assertions possible in a test.

```
#[derive(Debug, PartialEq)] // <- both required: PartialEq for `==`, Debug for the failure message
struct Order { id: u32, total_cents: u32 }

#[test]
fn totals_order_correctly() {
    let order = Order { id: 1, total_cents: 1999 };
    assert_eq!(order, Order { id: 1, total_cents: 1999 }); // needs PartialEq + Debug on Order
}
```

**Why this way:** without `Debug` the assertion doesn't compile at all
(not just on a runtime mismatch), since `assert_eq!` must be able to
print both sides on failure — the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
notes both `PartialEq` and `Debug` are needed for `assert_eq!`.

## Embedded Rust Notes

**Full support.** The built-in derives (`Debug`, `Clone`, `PartialEq`,
etc.) all work in `#![no_std]`. Worth noting: `#[derive(Debug)]`'s
formatting still goes through `core::fmt`, which has no built-in way to
print anywhere on a bare-metal target — embedded code typically routes
`Debug`/`Display` output through a crate like `defmt` (a
`no_std`-oriented, wire-efficient logging framework) rather than
`println!`, which requires `std`.

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
mechanical implementation, saving the boilerplate of writing it by hand —
for example, `#[derive(Debug, Clone, PartialEq)]` on a `Point { x: f64, y: f64 }`
struct generates all three implementations at once.

A derive only works if every field's own type already implements the
trait being derived — deriving `Clone` for a struct containing a
non-`Clone` field is a compile error, since there'd be no way to generate
a working implementation. This is also the mechanism behind
`serde`'s `#[derive(Serialize, Deserialize)]` — the same
generate-from-structure idea, extended by a third-party procedural macro
rather than the small fixed set of derives built into the compiler
itself, which is why the [serialization](../testing-tooling/serialization-serde.md)
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

## Explanation (Embedded)

The mechanism is unchanged under `#![no_std]` — every built-in derive
still generates the same mechanical, field-by-field implementation,
routed through `core::fmt`/`core::cmp`/`core::clone` rather than their
`std` re-exports, and still requires every field's own type to already
implement the trait being derived. What's worth deciding deliberately on
an embedded target is *which* traits to derive on *which* types, because
one of them has a cost that's easy to ignore on a hosted machine and hard
to ignore on a flash-constrained chip: `#[derive(Debug)]` generates
formatting code — matching every variant, writing every field name — that
occupies flash, and deriving it reflexively on every type in a large HAL
or driver crate can measurably inflate a release binary. See
[`#[derive(...)]`'s embedded
explanation](../../syntax/attributes/derive.md) for the full code-size
argument and the `#[cfg_attr(feature = "debug-impls", derive(Debug))]`
pattern used to keep it optional; the choice that belongs here, on the
concept page, is *when* a type genuinely needs `Debug`/`Clone`/`PartialEq`
at all — a hot, frequently-instantiated register-snapshot type is a poor
candidate for a reflexive `Debug` derive, while a rarely-constructed
configuration struct usually isn't worth worrying about either way.

## Basic usage example (Embedded)

```
#![no_std]

#[derive(Clone, Copy, PartialEq)] // <- routes through core::clone/core::cmp; Debug left out deliberately
pub struct SensorReading {
    pub raw_adc: u16,
}
```

## Best practices & deeper information (Embedded)

### Scenario: Implementing traits

A register-snapshot type read many times per second is exactly the kind
of type where reflexively deriving `Debug` alongside `Clone`/`PartialEq`
is worth reconsidering, since only the latter two are needed for the
driver's own logic to compare and copy readings.

```
#![no_std]

#[derive(Clone, Copy, PartialEq)] // <- needed: the driver compares and copies readings internally
pub struct StatusRegister {
    pub raw: u32,
}

// Debug omitted here on purpose — this type is read on every poll loop
// iteration, and its formatting code would occupy flash for a capability
// the release firmware never exercises.
```

**Why this way:** deriving exactly the traits a type's own logic needs —
and no more — keeps flash usage proportional to actual use, rather than
paying for `Debug`'s formatting code on a type instantiated and copied
constantly; see [`#[derive(...)]`'s embedded
explanation](../../syntax/attributes/derive.md) for the full code-size
reasoning behind treating `Debug` as opt-in on hot types.

### Scenario: Testing

A small, infrequently-constructed configuration type is a good candidate
for deriving `Debug` and `PartialEq` together, since host-run unit tests
need both to use `assert_eq!` — and this type's derive cost is paid once,
not on every poll loop iteration.

```
#![no_std]

#[derive(Debug, PartialEq)] // <- both needed for assert_eq! in host-run tests; cheap here: rarely constructed
pub struct UartConfig {
    pub baud_rate: u32,
    pub parity_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_config() {
        let cfg = UartConfig { baud_rate: 115_200, parity_enabled: false };
        assert_eq!(cfg, UartConfig { baud_rate: 115_200, parity_enabled: false });
    }
}
```

**Why this way:** `assert_eq!` still requires `Debug` + `PartialEq` on a
`no_std` target exactly as it does on a hosted one, and this type is
constructed rarely enough (at startup, from configuration) that its
`Debug` derive's flash cost is negligible — the code-size argument for
withholding `Debug` applies to hot, repeatedly-instantiated types, not to
every type in a crate uniformly.

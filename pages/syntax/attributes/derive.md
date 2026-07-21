---
title: "#[derive(...)]"
kind: attribute
embedded_support: full
groups: ["Traits & Derives", "Traits & Polymorphism"]
related_concepts: ["Derivable traits (Debug, Clone, PartialEq, …)", "Derive macros"]
related_syntax: [trait, impl]
see_also: ["Derivable traits (Debug, Clone, PartialEq, …)", "Derive macros"]
---

## Explanation

`#[derive(...)]`, placed directly above a `struct`, `enum`, or `union`
item, tells the compiler to generate one or more trait implementations
for that type automatically, based purely on its structure:
`#[derive(Debug, Clone, PartialEq)]` above a struct generates a `Debug`
impl, a `Clone` impl, and a `PartialEq` impl in one line, without a
single line of hand-written `impl` code.

Compared to writing `impl Debug for Point { ... }` by hand, a derive:

- **Generates mechanically, field-by-field** — it compares/prints/clones
  every field, in the order they're declared, with no way to skip or
  customize individual fields from the attribute alone. The moment a
  field needs special treatment (excluded from equality, formatted
  differently), the derive is the wrong tool — write the `impl` by hand
  instead.
- **Requires every field's own type to already implement the trait** — a
  struct containing a field whose type doesn't implement `Clone` cannot
  itself derive `Clone`; the compiler reports the missing bound as an
  error at the derive site, not somewhere deep in generated code.
- **Stays in sync automatically as fields change** — adding or removing a
  field updates the generated impl the next time the code compiles, with
  no hand-written code to remember to update.

`#[derive(...)]` is the *call site* of the broader **derive macro**
mechanism: each name inside the parentheses (`Debug`, `Clone`, `Serialize`,
...) is either one of a small set of traits the compiler derives as a
built-in intrinsic, or a trait provided by a crate along with its own
`#[proc_macro_derive]` function that generates the impl the same way a
built-in derive does. This page covers the attribute itself; see
[Derivable traits](../../concepts/traits-polymorphism/derivable-traits.md)
for which traits the compiler derives natively and their exact rules, and
[Derive macros](../../concepts/macros-metaprogramming/derive-macros.md)
for how a crate builds its *own* derivable trait using this same
attribute syntax.

## Usage examples

### Deriving common traits mechanically

```
#[derive(Debug, Clone, PartialEq)] // <- generates Debug, Clone, and PartialEq impls mechanically
struct Point { x: f64, y: f64 }

let a = Point { x: 1.0, y: 2.0 };
let b = a.clone();
println!("{:?} {}", b, a == b);
```

### Implementing traits

Reaching for `#[derive(...)]` first, and only dropping to a hand-written
`impl` once the mechanical, field-by-field behavior stops being correct,
keeps a type's trait implementations both terse and honest about when
something custom is actually happening.

```
#[derive(Debug, Clone)] // <- straightforward mechanical impls: nothing custom needed here
struct SensorReading {
    celsius: f64,
    measured_at: u64,
}

// PartialEq needs to be hand-written: measured_at shouldn't count toward equality
impl PartialEq for SensorReading {
    fn eq(&self, other: &Self) -> bool {
        self.celsius == other.celsius
    }
}
```

`#[derive(PartialEq)]` would compare `measured_at` too,
which isn't the intended meaning of equality here — the
[API Guidelines' C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html)
still recommends implementing `PartialEq`, just not necessarily via
derive, whenever the mechanical field-by-field behavior isn't what's
wanted.

### Testing

`assert_eq!` requires both `PartialEq` (to compare the two sides) and
`Debug` (to print them in the failure message) — deriving both together
is what makes an ordinary equality assertion in a test compile at all.

```
#[derive(Debug, PartialEq)] // <- both needed: PartialEq for `==`, Debug for the panic message on mismatch
struct Order { id: u32, total_cents: u32 }

#[test]
fn totals_order_correctly() {
    let order = Order { id: 1, total_cents: 1999 };
    assert_eq!(order, Order { id: 1, total_cents: 1999 });
}
```

Without `#[derive(Debug)]`, this fails to *compile*,
not just to pass — `assert_eq!` requires `Debug` on both sides to build
its failure message, as the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
notes; deriving both traits in one attribute is the standard way to make
a struct or enum usable in `assert_eq!` at all.

## Explanation (Embedded)

Under `#![no_std]`, `#[derive(...)]` for anything built purely on `core`
traits — `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`,
`PartialOrd`, `Ord` — works exactly as described above: the same
mechanical, field-by-field generation, the same requirement that every
field's type already implement the trait, routed through `core::fmt` /
`core::cmp` / `core::clone` instead of their `std` re-exports. Nothing
about the attribute itself changes.

Where embedded code diverges in practice, not in mechanism, is
`#[derive(Debug)]` specifically: a `Debug` impl's formatting code
(matching on every variant, writing out every field name) is generated
code that occupies flash, and on a part with a genuinely tight flash
budget, deriving `Debug` on every type in a large HAL — especially large
enums or deeply nested register structs — can noticeably inflate binary
size for a capability a release firmware image may never actually
invoke. It's common to see `Debug` derives feature-gated
(`#[cfg_attr(feature = "debug-impls", derive(Debug))]`) or skipped
entirely on the hottest, most repeated types in a crate for exactly this
code-size reason, while still deriving it freely on types that are rare
or only used during development.

## Usage examples (Embedded)

### Deriving core-only traits identically under no_std

```
#![no_std]

#[derive(Debug, Clone, Copy, PartialEq)] // <- routes through core::fmt/core::clone/core::cmp, not std
pub struct SensorReading {
    pub raw_adc: u16,
}
```

### Feature-gating Debug to keep its code out of the flash budget by default

```
#[cfg_attr(feature = "debug-impls", derive(Debug))] // <- Debug's formatting code only ships when explicitly opted into
#[derive(Clone, Copy)]
pub struct RegisterSnapshot {
    pub value: u32,
}
```

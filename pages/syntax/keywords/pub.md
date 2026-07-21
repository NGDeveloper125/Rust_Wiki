---
title: "pub"
kind: keyword
embedded_support: full
groups: ["Modules & Visibility", "Modules, Crates & Visibility"]
related_concepts: ["Visibility & privacy"]
related_syntax: [mod, crate, super, use]
see_also: [mod, crate, super]
---

## Explanation

`pub` widens an item's default visibility, which is otherwise private —
reachable only from the module that defines it and that module's
descendants. Written alone before an item (`pub fn`, `pub struct`,
`pub mod`, `pub use`, …), it removes that restriction entirely: the item
is visible from anywhere that can reach the module defining it, including
outside the crate, provided the module path leading to it is itself
public.

Four scoped forms narrow that ceiling instead of removing it outright:

- **`pub`** — no restriction beyond ordinary module-tree reachability.
- **`pub(crate)`** — visible anywhere in the same crate, never outside
  it. The most common scoped form: it shares an item across a crate's own
  modules without adding it to the crate's public API.
- **`pub(super)`** — visible to the parent module (and transitively,
  anywhere the parent's own visibility already reaches). Shorthand for
  `pub(in <parent path>)`.
- **`pub(in some::path)`** — visible only within `some::path` and that
  path's descendant modules. `some::path` must name an ancestor module of
  the item being marked — this form can only *restrict* how far down an
  already-reachable subtree the boundary is drawn, never grant visibility
  to a module that couldn't already see the item. `pub(crate)` and
  `pub(super)` are convenience shorthands for the two most common
  ancestors: the crate root, and the immediate parent.

On a struct, `pub` (in any form) applies per field, not automatically to
the whole struct: `pub struct Point { pub x: i32, y: i32 }` makes the
struct name itself visible while `x` is public and `y` stays private. See
[Visibility & privacy](../../concepts/modules-crates-visibility/visibility-and-privacy.md)
for the design rationale behind this kind of partial exposure — this page
covers the grammar of `pub` and its scoped forms.

## Usage examples

### Making a struct and field visible outside its module

```
mod billing {
    pub struct Invoice {   // <- `pub`: visible outside the `billing` module
        pub total_cents: u64,
    }
}

let invoice = billing::Invoice { total_cents: 4200 };
```

### Designing a public API

A cache crate splits its storage and eviction logic into separate
modules that need to share an internal entry type — `pub(crate)` lets them
do that without the type ever becoming part of the crate's public API.

```
pub mod api {                          // <- plain `pub`: visible from outside the crate entirely
    pub(crate) mod internal {          // <- `pub(crate)`: visible anywhere in this crate, not outside it
        pub(super) fn raw_encode(bytes: &[u8]) -> Vec<u8> {
            // <- `pub(super)`: visible only to `api`, the parent module
            bytes.to_vec()
        }

        pub(in crate::api) fn checksum(bytes: &[u8]) -> u32 {
            // <- `pub(in path)`: visible only within `api` and its descendants
            bytes.iter().map(|b| *b as u32).sum()
        }
    }

    pub fn encode(bytes: &[u8]) -> Vec<u8> {
        internal::raw_encode(bytes) // reachable: `api` is `internal`'s parent module
    }
}
```

Each scoped form draws the visibility boundary at
exactly the module that needs it — `pub(crate)` shares `internal` across
the crate, `pub(super)`/`pub(in path)` narrow individual functions further
still — so nothing here is visible any wider than the code that actually
uses it requires, matching the general shrink-visibility-first stance in
the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html).

### Validating input

A `Temperature` type must never represent a value below absolute zero, so
its field stays private and `pub` is applied only to the constructor and
the read accessor, not the field itself.

```
pub struct Temperature {
    celsius: f64,          // private: no outside code can set an invalid value directly
}

impl Temperature {
    pub fn from_celsius(value: f64) -> Result<Self, &'static str> {
        // <- `pub`: the only public entry point that can construct one
        if value < -273.15 {
            return Err("temperature below absolute zero");
        }
        Ok(Temperature { celsius: value })
    }

    pub fn celsius(&self) -> f64 { // <- `pub`: read-only access, no bypass of the check above
        self.celsius
    }
}
```

Applying `pub` to the constructor and getter but not the
field means "never below absolute zero" is enforced by the type itself,
which is exactly what the
[API Guidelines' C-STRUCT-PRIVATE](https://rust-lang.github.io/api-guidelines/future-proofing.html#structs-have-private-fields-c-struct-private)
item recommends private fields for.

## Embedded Rust Notes

**Full support.** Visibility is enforced entirely at compile time with no
runtime cost, so every form of `pub` behaves identically under
`#![no_std]`. Embedded HAL crates rely on the scoped forms just as much as
hosted code — commonly `pub(crate)` for register-access helpers shared
between a driver's own submodules but never exposed to firmware code
using the driver.

---
title: "super"
kind: keyword
embedded_support: full
groups: ["Modules & Visibility", "Modules, Crates & Visibility"]
related_concepts: [Modules, "Visibility & privacy"]
related_syntax: [crate, mod, pub, "::"]
see_also: [crate, mod]
---

## Explanation

`super` used as the first segment of a path names the parent of the
module writing it: `super::helper()` reaches an item one level up in the
module tree. Like [`crate`](crate.md) and `self`, it's a path-leading
keyword rather than an ordinary identifier — it appears as a path's first
segment, and can be repeated to keep climbing: `super::super::helper()`
reaches the grandparent module.

Chaining `super::super::...` is legal, but each repetition ties the path
to one specific position in the module tree — if a module moves, every
chain referencing it needs updating to match. More than one level of
`super` chaining is usually a sign the module tree itself could be
flattened, or that the referenced item would read more clearly reached
with an explicit [`use`](use.md) or a `crate::`-rooted path instead; it's
a style note, not a hard rule.

## Usage examples

### Reaching a parent module's function with `super::`

```
fn shared_helper() -> u32 { 7 }

mod util {
    pub fn double() -> u32 {
        super::shared_helper() * 2 // <- `super::` reaches the parent module's item
    }
}
```

### Designing a public API

A `discounts` submodule reaches a private helper defined in its parent
`pricing` module via `super::`, without that helper ever needing to be
made `pub`.

```
pub mod pricing {
    fn base_shipping_cents() -> u32 { // private: reachable from `pricing` and its descendants
        499
    }

    pub mod discounts {
        pub fn free_shipping_threshold_cents() -> u32 {
            super::base_shipping_cents() * 20
            // <- `super::` reaches `pricing`, the parent module
        }
    }
}
```

Because `discounts` is a descendant of `pricing`, it
already has access to `pricing`'s private items — see
[Visibility & privacy](../../concepts/modules-crates-visibility/visibility-and-privacy.md)
for why that descendant access exists — so `super::` is simply how that
existing reachability gets spelled out as a path, with no need to widen
`base_shipping_cents`'s visibility just to let a submodule use it.

## Embedded Rust Notes

**Full support.** `super` is a compile-time path prefix with no runtime
representation, so it resolves identically under `#![no_std]`.

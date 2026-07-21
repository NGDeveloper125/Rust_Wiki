---
title: "#[doc = \"...\"]"
kind: attribute
embedded_support: full
groups: ["Testing & Tooling"]
related_concepts: ["Doc tests"]
related_syntax: ["///", "//!"]
see_also: ["///", "//!"]
---

## Explanation

`#[doc = "..."]` is the attribute form every doc comment desugars into —
an outer [`///`](../comments/outer-line-doc-comment.md) comment becomes an
outer `#[doc = "..."]` attribute on the item that follows it, and an inner
[`//!`](../comments/inner-line-doc-comment.md) comment becomes an inner
`#![doc = "..."]` attribute on the enclosing module or crate. Both of
those pages already state the desugaring and cover writing documentation
prose day to day; this page is about the forms that have **no
comment-syntax equivalent at all** — the `#[doc(...)]` family, where the
parentheses hold a meta item rather than a string.

- **`#[doc(hidden)]`** excludes an item from rustdoc's generated output
  entirely, even though the item is `pub` and would otherwise appear. This
  is for items that must be public — often because a macro expansion
  needs to reference them from the calling crate — but aren't meant to be
  part of the API a human ever reads about or calls directly. Hiding the
  item doesn't change its visibility or callability, only whether
  `cargo doc` renders a page for it.
- **`#[doc(alias = "...")]`** registers an additional search term for an
  item in rustdoc's generated search index, without changing the item's
  actual name. This helps readers who search for a term the API doesn't
  literally use — a method named `len` might carry
  `#[doc(alias = "size")]` so a reader searching "size" still finds it.
- **`#[doc(inline)]`**, placed on a `pub use` re-export, forces rustdoc to
  render the re-exported item's **full documentation** at the re-export's
  location, instead of the default behavior of showing just a short
  "Re-export of ..." link pointing back to where the item was originally
  defined. This matters when a crate's public-facing module structure
  differs from its internal one — a type defined deep in a private
  internal module, re-exported at the crate root, reads better with its
  complete docs inlined at the root than as a link the reader has to
  follow.

## Basic usage example

```
#[doc(alias = "size")] // <- makes this method findable in rustdoc search under "size" too
pub fn len(&self) -> usize {
    self.items.len()
}
```

## Best practices & deeper information

### Scenario: Documenting an API

A derive macro generates a helper trait implementation that references an
internal type the macro needs public for the expansion to compile, but
that type was never meant to be part of the crate's documented API —
`#[doc(hidden)]` keeps it out of the generated docs without making it
private, which would break the macro-generated code that needs to name it.

```
// Generated (conceptually) by a derive macro on some public type:
#[doc(hidden)] // <- must stay pub for macro-generated code to reference it, but isn't real API surface
pub struct SensorReadingFieldNames;

impl SensorReadingFieldNames {
    #[doc(hidden)]
    pub fn names() -> &'static [&'static str] {
        &["id", "celsius"]
    }
}
```

**Why this way:** a macro-generated support type frequently has to be
`pub` purely so the expanded code compiles in the caller's crate, even
though no human is meant to use it directly — the
[rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html#hidden)
documents `#[doc(hidden)]` as exactly this "publicly reachable, not
publicly documented" escape hatch, distinct from actually making the item
private (which would break the very macro expansion it exists to support).

### Scenario: Designing a public API

A crate's real implementation lives in a private internal module, but the
type is re-exported at the crate root as the officially supported way to
use it — `#[doc(inline)]` makes the root-level documentation page show
the type's full docs directly, rather than sending readers on a detour
through an internal module they were never meant to know exists.

```
mod internal_pricing_engine {
    /// Computes a discounted total from a price and a percentage off.
    pub struct DiscountCalculator {
        pub percent_off: u8,
    }

    impl DiscountCalculator {
        pub fn apply(&self, price_cents: u32) -> u32 {
            price_cents - (price_cents * self.percent_off as u32 / 100)
        }
    }
}

#[doc(inline)] // <- shows DiscountCalculator's full docs here, not just a link to internal_pricing_engine
pub use internal_pricing_engine::DiscountCalculator;
```

**Why this way:** without `#[doc(inline)]`, rustdoc's default re-export
rendering is a short link back to `internal_pricing_engine`, which is a
confusing detour for a module that isn't meant to be part of the public
API surface at all — the
[rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html#inline)
documents `inline` as the way to make a re-export's documentation page
self-contained when the internal path it came from isn't meant to be
navigated to directly.

## Embedded Rust Notes

**Full support.** `#[doc(...)]` generation is a `cargo doc`/rustdoc-time
concern with no dependency on `std` or an allocator, so it works
identically for `#![no_std]` crates — a HAL crate's macro-generated
register-access internals are exactly the kind of thing `#[doc(hidden)]`
is reached for in embedded codebases specifically.

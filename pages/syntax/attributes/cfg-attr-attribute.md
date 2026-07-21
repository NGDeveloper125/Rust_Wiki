---
title: "#[cfg_attr(...)]"
kind: attribute
embedded_support: full
groups: ["Modules, Crates & Visibility"]
related_concepts: ["Crates", "Cargo & Cargo.toml"]
related_syntax: ["#[cfg(...)]", "#[meta] / #![meta]", "#[derive(...)]"]
see_also: ["#[cfg(...)]"]
---

## Explanation

`#[cfg_attr(condition, attribute)]` conditionally applies **another
attribute** to the item it decorates: if `condition` holds (using exactly
the same condition grammar as [`#[cfg(...)]`](cfg-attribute.md) —
`test`, `target_os = "..."`, `all(...)`/`any(...)`/`not(...)`, and so on),
the item behaves as if `attribute` had been written directly on it; if the
condition doesn't hold, `attribute` is never applied at all, as though
this line weren't there. Multiple attributes can be requested from one
`cfg_attr` by comma-separating them after the condition:
`#[cfg_attr(test, derive(Debug, PartialEq))]`.

The item itself is never removed by `cfg_attr` — that's what
`#[cfg(...)]` does. `cfg_attr` only ever adds or withholds an *attribute*
conditionally; the item it's attached to compiles in every configuration
either way.

**Why this exists instead of two `#[cfg(...)]` blocks.** Without
`cfg_attr`, getting an extra derive only under one condition would require
duplicating the entire item — one copy gated `#[cfg(test)]` with `#[derive(Debug,
PartialEq)]` added, and a second, otherwise-identical copy gated
`#[cfg(not(test))]` without that derive at all.

Any change to `Reading`'s fields now has to be made in two places at once
and kept in sync by hand. `#[cfg_attr(test, derive(Debug, PartialEq))]` on
a single copy of the struct says the same thing without the duplication —
the item is written once, and only the extra attribute is conditional.

## Basic usage example

```
#[cfg_attr(test, derive(Debug))] // <- derives Debug only when compiling for `cargo test`
struct Reading {
    celsius: f64,
}
```

## Best practices & deeper information

### Scenario: Testing

A domain type doesn't need `Debug` in ordinary builds, but test code
constantly wants it for `assert_eq!` failure messages and `dbg!` output —
`cfg_attr` adds the derive only where it's actually needed, without a
second, duplicated struct definition.

```
#[cfg_attr(test, derive(Debug, PartialEq))] // <- Debug/PartialEq only exist in test builds
pub struct OrderTotal {
    cents: u64,
}

impl OrderTotal {
    pub fn new(cents: u64) -> Self {
        OrderTotal { cents }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_compare_equal() {
        // <- both derives from cfg_attr are what make assert_eq! compile here
        assert_eq!(OrderTotal::new(500), OrderTotal::new(500));
    }
}
```

**Why this way:** `assert_eq!` needs both `Debug` (to print the failure
message) and `PartialEq` (to compare) on `OrderTotal`, but a published
library's normal build gains nothing from either derive on a purely
internal amount type — `cfg_attr` keeps both traits scoped to exactly the
builds that need them, which the
[Rust Reference](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute)
documents as `cfg_attr`'s purpose: attaching an attribute conditionally
without writing the item itself twice.

### Scenario: Designing a public API

A struct needs `#[repr(C)]` only when built for a specific target that
crosses an FFI boundary, while other targets are free to let the compiler
choose its own layout — `cfg_attr` keeps the one struct definition shared
across both cases.

```
#[cfg_attr(target_os = "linux", repr(C))] // <- repr(C) applied only when targeting Linux
pub struct FrameHeader {
    pub version: u8,
    pub flags: u8,
}
```

**Why this way:** writing the struct out twice — once `#[repr(C)]` gated
to Linux, once without a repr gated to everything else — would risk the
two copies' field lists drifting apart over time; `cfg_attr` keeps exactly
one field list as the single source of truth while only the layout
attribute varies by target.

## Embedded Rust Notes

**Full support.** `cfg_attr` is a pure compile-time mechanism with no
allocator or OS dependency, and is common in embedded crates that support
several chip families from one codebase — for example applying
`#[cfg_attr(feature = "defmt", derive(defmt::Format))]` so a type derives
a `no_std`-friendly logging format only when the crate's `defmt` feature
is enabled, without a second copy of the type for builds that don't use
it.

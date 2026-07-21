---
title: "#[cfg_attr(...)]"
kind: attribute
embedded_support: full
groups: ["Conditional Compilation", "Modules, Crates & Visibility"]
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

## Usage examples

### Conditionally deriving a trait for test builds

```
#[cfg_attr(test, derive(Debug))] // <- derives Debug only when compiling for `cargo test`
struct Reading {
    celsius: f64,
}
```

### Testing

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

`assert_eq!` needs both `Debug` (to print the failure
message) and `PartialEq` (to compare) on `OrderTotal`, but a published
library's normal build gains nothing from either derive on a purely
internal amount type — `cfg_attr` keeps both traits scoped to exactly the
builds that need them, which the
[Rust Reference](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute)
documents as `cfg_attr`'s purpose: attaching an attribute conditionally
without writing the item itself twice.

### Designing a public API

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

Writing the struct out twice — once `#[repr(C)]` gated
to Linux, once without a repr gated to everything else — would risk the
two copies' field lists drifting apart over time; `cfg_attr` keeps exactly
one field list as the single source of truth while only the layout
attribute varies by target.

## Explanation (Embedded)

`cfg_attr` is the compile-time-only combinator paired with
[`#[cfg(...)]`](cfg-attribute.md) that embedded crates lean on constantly
for a slightly different job: not choosing which module exists, but
choosing which *attribute* a shared item gets. The most common embedded
case is conditionally deriving a `no_std`-friendly logging trait: a HAL's
error type wants `#[derive(defmt::Format)]` when the crate's `defmt`
feature is enabled (for wire-efficient logging over RTT/probe-rs), but a
build that doesn't want that dependency at all shouldn't pull it in —
`#[cfg_attr(feature = "defmt", derive(defmt::Format))]` on the type adds
that derive only when the feature is on, while exactly one struct
definition serves both builds. A second common case pairs `cfg_attr` with
`cortex-m-rt`'s `#[interrupt]`/`#[exception]` attributes: a handler
function sometimes needs a target- or feature-conditional
`#[link_section = "..."]` alongside its interrupt registration, so the
same function body gets the extra attribute only on the
target/feature combination that needs it, instead of two near-duplicate
copies of the handler.

## Usage examples (Embedded)

### Deriving a no_std logging trait only when the defmt feature is enabled

```
#[cfg_attr(feature = "defmt", derive(defmt::Format))] // <- adds defmt's Format derive only when the "defmt" feature is on
#[derive(Debug, Clone, Copy)]
pub enum SensorError {
    Timeout,
    BusFault,
}
```

### Relocating an interrupt handler to RAM only on a feature-gated build

```
#[cortex_m_rt::interrupt]
#[cfg_attr(feature = "ram-handlers", link_section = ".data.TIM2")] // <- relocated to RAM only when that feature is on
fn TIM2() {
    // handle the timer-2 interrupt
}
```

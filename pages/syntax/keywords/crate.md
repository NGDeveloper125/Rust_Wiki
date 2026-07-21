---
title: "crate"
kind: keyword
embedded_support: full
groups: ["Modules & Visibility", "Modules, Crates & Visibility"]
related_concepts: [Crates]
related_syntax: [mod, super, pub, "::"]
see_also: [mod, super]
---

## Explanation

`crate` used as the first segment of a path always names the current
crate's root module: `crate::helper()` refers to the same item whether
it's written at the crate root or ten modules deep. It's one of a small
set of keywords that pick a path's starting point explicitly — alongside
`self` (the current module) and [`super`](super.md) (the parent module) —
instead of relying on `use`-imported or item-relative name resolution.

`crate` cannot be used as an ordinary identifier and only ever appears as
the leading segment of a path (`crate::foo::Bar`, never `foo::crate::Bar`
or a standalone value). See
[Crates](../../concepts/modules-crates-visibility/crates.md) for what a
crate is and why the crate boundary matters architecturally (it's also
the ceiling `pub(crate)` visibility draws); this page is about writing the
`crate::` path prefix itself.

## Usage examples

### Referring to the crate root with `crate::`

```
fn format_timestamp(seconds: u64) -> String { // a crate-root helper
    format!("{seconds}s")
}

mod logging {
    pub fn log(message: &str) {
        println!("[{}] {}", crate::format_timestamp(0), message);
        // <- `crate::` names this crate's root from inside `logging`
    }
}
```

### Designing a public API

A reporting module nested three levels deep still needs the crate root's
`current_user_id` helper — `crate::` reaches it directly, regardless of
how deep `csv` happens to sit in the module tree.

```
pub fn current_user_id() -> u32 { // crate-root helper, reachable via `crate::` from anywhere
    42
}

pub mod reports {
    pub mod export {
        pub mod csv {
            pub fn header_row() -> String {
                format!("user_id={}", crate::current_user_id())
                // <- `crate::` reaches the root from three levels deep
            }
        }
    }
}
```

`crate::current_user_id()` names the same path no
matter which module writes it, so moving `csv` to a different nesting
depth later doesn't change how it refers back to the root — unlike a
relative `super::super::...` chain, which would need editing every time a
module moves.

## Explanation (Embedded)

`crate::` resolves identically under `#![no_std]` — a compile-time path
prefix naming the current crate's root module, with no runtime
representation on any target. It's exactly as useful in a HAL crate's
typical layout as anywhere else: a peripheral-access module nested a few
levels under the crate root (a driver's `gpio::pin` submodule, say)
reaches the generated register definitions with `crate::pac::GPIOA`
regardless of how deep the calling module sits, rather than working out
a relative `super::super::...` path to the `pac` module every time the
driver's own module tree gets reorganized.

## Usage examples (Embedded)

### Reaching the generated register module from a nested driver submodule

```
// src/lib.rs
#![no_std]

pub mod pac;   // generated register definitions, at the crate root

pub mod gpio {
    pub mod pin {
        pub fn set_high(pin_number: u8) {
            unsafe {
                crate::pac::GPIOA.odr.modify(|_, w| w.bits(1 << pin_number));
                // <- `crate::` reaches `pac` from two levels deep in `gpio::pin`
            }
        }
    }
}
```

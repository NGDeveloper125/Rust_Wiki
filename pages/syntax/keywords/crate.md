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

## Embedded Rust Notes

**Full support.** `crate` is a compile-time path prefix with no runtime
representation, so it resolves identically in a `#![no_std]` crate —
`crate::` still names that crate's own root module regardless of target.

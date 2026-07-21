---
title: "#[macro_export] / #[macro_use]"
kind: attribute
embedded_support: full
groups: ["Macros", "Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)"]
related_syntax: ["macro_rules!", "use", "pub"]
see_also: ["Declarative macros (macro_rules!)"]
---

## Explanation

`#[macro_export]`, placed directly above a `macro_rules!` definition,
makes that macro usable outside the crate that defines it. This
attribute is necessary because a `macro_rules!` macro is **not**
implicitly crate-public the way other items become public with `pub` —
nesting a macro definition inside a `pub mod` doesn't export it; without
`#[macro_export]` it stays visible only inside the defining crate.

Its second effect is relocation: an exported macro is placed at the
**crate root**, regardless of which module it was textually defined in.
External code refers to it as `crate_name::macro_name!(...)`, or imports
it with `use crate_name::macro_name;`, never through the module path the
`macro_rules!` block actually sits in.

Because an exported macro's expansion is spliced into the *caller's*
crate, a bare path inside its transcriber pointing at another item in
the defining crate would try (and typically fail) to resolve in the
caller's crate instead. The special `$crate` metavariable exists exactly
for this: `$crate::helper_fn()` always resolves to the defining crate's
`helper_fn`, no matter which crate invokes the macro.

`#[macro_use]` is the older, pre-2018-edition mechanism for bringing
another scope's macros into view: placed above an `extern crate` item,
it imports all of that crate's exported macros unqualified; placed above
a `mod` item, it does the same for that module's macros in the enclosing
scope. Since the 2018 edition made macros full members of the module
system, an exported macro can simply be brought in with an ordinary
`use crate_name::macro_name;`, the same way any other item is imported —
clearer (it names one specific macro), and composable with the rest of
the import system (`as`-renaming, glob imports). `#[macro_use]` still
compiles and still appears in edition-2015-era code and some crates'
older examples, but it's a legacy form now; new code should prefer
`use`.

## Basic usage example

```
#[macro_export] // <- opts this macro into being usable from other crates
macro_rules! log_event {
    ($msg:expr) => { println!("[event] {}", $msg) };
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A logging crate exports a formatting macro so downstream crates get the
same convenience `println!`/`format!` already give them, without needing
a logger instance passed around.

```
// crate "telemetry", src/lib.rs
#[macro_export] // <- required: without it, log_event! would stay private to this crate
macro_rules! log_event {
    ($($arg:tt)*) => { // <- forwards arbitrary format arguments, the same way println! does
        println!("[telemetry] {}", format!($($arg)*))
    };
}

// downstream crate, Cargo.toml: telemetry = { path = "../telemetry" }
use telemetry::log_event; // <- imported like any ordinary item, thanks to 2018-edition macro scoping

fn start() {
    log_event!("sensor {} online", 7); // <- resolves via the crate root, not telemetry's internal module path
}
```

**Why this way:** macro visibility doesn't follow ordinary item
visibility rules, so `#[macro_export]` is mandatory for any macro meant
to be used outside its crate; importing the result with a plain `use`,
rather than `#[macro_use]`, is exactly the improvement the
[Rust Reference's macro scoping rules](https://doc.rust-lang.org/reference/macros-by-example.html#path-based-scope)
describe as the 2018-edition path-based alternative to the older,
attribute-based import.

## Embedded Rust Notes

**Full support.** Both attributes are resolved entirely at compile time
as part of name resolution, with no runtime footprint — identical under
`#![no_std]`. Embedded HAL and `heapless`-adjacent crates commonly use
`#[macro_export]` to let firmware crates use their register- or
pin-definition macros; `#[macro_use]` still turns up in older,
edition-2015-pinned embedded example code, but current crates use `use`.

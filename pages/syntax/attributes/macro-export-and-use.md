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

## Usage examples

### Exporting a macro so other crates can use it

```
#[macro_export] // <- opts this macro into being usable from other crates
macro_rules! log_event {
    ($msg:expr) => { println!("[event] {}", $msg) };
}
```

### Designing a public API

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

Macro visibility doesn't follow ordinary item
visibility rules, so `#[macro_export]` is mandatory for any macro meant
to be used outside its crate; importing the result with a plain `use`,
rather than `#[macro_use]`, is exactly the improvement the
[Rust Reference's macro scoping rules](https://doc.rust-lang.org/reference/macros-by-example.html#path-based-scope)
describe as the 2018-edition path-based alternative to the older,
attribute-based import.

## Explanation (Embedded)

Both attributes work exactly as described above under `#![no_std]` —
they're resolved entirely during name resolution, before any code
generation happens, so there's no runtime or allocator dependency to
speak of. The pattern shows up concretely in the embedded ecosystem in
register-definition and peripheral-access crates: a low-level crate that
defines a `define_register!`-style `macro_rules!` helper for declaring
memory-mapped register accessors marks it `#[macro_export]` so downstream
HAL crates — and the application firmware built on top of them — can
pull the macro in with an ordinary `use`, the same way they'd import any
other item, rather than needing the macro's logic duplicated in every
consuming crate. `#[macro_use]` still turns up in older peripheral-access
crates generated before the 2018 edition's path-based macro scoping, but
current `svd2rust` output and hand-written HALs use `use`.

## Usage examples (Embedded)

### Exporting a register-definition macro for HAL crates to consume

```
// crate "chip-pac", src/lib.rs
#[macro_export] // <- required: without it, define_register! stays private to chip-pac
macro_rules! define_register {
    ($name:ident, $addr:expr) => {
        pub struct $name;
        impl $name {
            pub unsafe fn read() -> u32 {
                core::ptr::read_volatile($addr as *const u32)
            }
        }
    };
}

// downstream HAL crate, Cargo.toml: chip-pac = { path = "../chip-pac" }
use chip_pac::define_register; // <- imported like any ordinary item

define_register!(GpioaIdr, 0x4002_0010);
```

---
title: "#[cfg(...)]"
kind: attribute
embedded_support: full
groups: ["Conditional Compilation", "Modules, Crates & Visibility"]
related_concepts: ["Crates", "Cargo & Cargo.toml"]
related_syntax: ["#[meta] / #![meta]", "#[cfg_attr(...)]", "cfg!"]
see_also: ["#[cfg_attr(...)]", "cfg!"]
---

## Explanation

`#[cfg(condition)]`, placed as an outer attribute above an item (or `#![cfg(condition)]`
as an inner attribute at the top of a module), tells the compiler to
**entirely remove that item from the compiled output** unless `condition`
holds. This isn't a runtime `if` — the item's tokens are discarded before
type-checking ever sees them when the condition is false, the same way as
if that code had never been typed at all. A `#[cfg(target_os = "windows")]`
function does not exist, in any form, in a binary built for Linux.

This is the load-bearing difference from the **expression-position**
[`cfg!(...)` macro](../macros/cfg-macro.md): `cfg!(...)` compiles *every*
branch of the surrounding code and evaluates to a plain `bool` at
compile time, which an ordinary `if` then branches on at runtime — both
sides of that `if` must still type-check and compile even though only one
ever runs. `#[cfg(...)]`, by contrast, never lets the false branch's code
exist at all: it can reference types, functions, or entire dependencies
that don't even compile on the excluded platform, because the compiler
never attempts to compile them. Reach for `#[cfg(...)]` to remove an item
outright (a whole module, an entire function, a struct field); reach for
`cfg!(...)` only when both branches are valid, compilable code and the
choice is genuinely a runtime one.

**Common conditions**, combined freely inside the parentheses:

- `target_os = "linux"` / `"windows"` / `"macos"` — the target operating
  system.
- `target_arch = "x86_64"` / `"aarch64"` / `"wasm32"` — the target CPU
  architecture.
- `feature = "some-feature"` — a Cargo feature flag declared in
  `Cargo.toml`'s `[features]` table, on or off depending on how the crate
  was built.
- `test` — true only when compiling for `cargo test` (this is exactly the
  mechanism [`#[cfg(test)]`](../../concepts/testing-tooling/unit-tests.md)
  relies on to keep a test module out of ordinary builds).
- `debug_assertions` — true in a standard debug build, false in a
  `--release` build; the same flag `debug_assert!` is gated on.

**Combinators** compose conditions like boolean logic, but as nested meta
items rather than operators: `all(a, b)` is AND (every condition must
hold), `any(a, b)` is OR (at least one must hold), and `not(a)` is
negation — for example, `#[cfg(all(unix, not(target_os = "macos")))]`
selects Unix-like targets excluding macOS specifically.

## Usage examples

### Compiling different function bodies per target OS

```
#[cfg(target_os = "linux")] // <- this whole function is removed from the build on any other OS
fn platform_name() -> &'static str {
    "linux"
}

#[cfg(not(target_os = "linux"))] // <- compiled only when the item above is not
fn platform_name() -> &'static str {
    "not linux"
}
```

### Designing a public API

A driver crate supports several platforms, each with a genuinely
different implementation module — `#[cfg(...)]` on the `mod` declarations
is what keeps only the one relevant module's code in the compiled
artifact, so platform-specific dependencies for other platforms never
even need to compile.

```
#[cfg(target_os = "linux")] // <- entire module compiled only when targeting Linux
mod linux_backend;

#[cfg(target_os = "windows")] // <- entire module compiled only when targeting Windows
mod windows_backend;

#[cfg(target_os = "linux")]
use linux_backend::open_serial_port;

#[cfg(target_os = "windows")]
use windows_backend::open_serial_port;

pub fn connect() {
    let _port = open_serial_port();
}
```

Each backend module can freely call platform-specific
APIs that simply don't exist (and wouldn't compile) on the other
platform, because `#[cfg(...)]` guarantees the excluded module's code is
never handed to the compiler at all — the
[Rust Reference](https://doc.rust-lang.org/reference/conditional-compilation.html)
documents `cfg` attributes as operating before macro expansion and
type-checking for exactly this reason, unlike a runtime `if` which would
require both backends to compile unconditionally.

### Working with collections

A struct carries an extra diagnostic field only in debug builds, keeping
the release build's memory layout and binary size free of data nobody
reads outside development.

```
struct RequestLog {
    endpoint: String,
    #[cfg(debug_assertions)] // <- field exists only in debug builds
    raw_headers: Vec<String>,
}

impl RequestLog {
    fn new(endpoint: String) -> Self {
        RequestLog {
            endpoint,
            #[cfg(debug_assertions)] // <- initializer must match: present exactly when the field is
            raw_headers: Vec::new(),
        }
    }
}
```

`#[cfg(...)]` on an individual struct field removes it
from the type's layout entirely in builds where the condition is false,
so a release build pays zero size or allocation cost for a field that
exists purely to help debugging — every place the field is named
(declaration and every constructor) needs the matching `#[cfg(...)]`, or
the mismatched builds fail to compile.

## Embedded Rust Notes

**Full support.** `#[cfg(...)]` is the primary mechanism embedded crates
use to support many different microcontroller targets from one codebase —
`target_arch = "arm"`, a chip-family Cargo feature (`feature = "stm32f4"`),
or a vendor HAL's own `cfg`-gated pin/peripheral modules all rely on it to
keep unsupported targets' code from ever being compiled, since much of
that code (raw register addresses, chip-specific intrinsics) wouldn't
even type-check against a different target's memory map.

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

## Explanation (Embedded)

`#[cfg(...)]` is arguably the single most load-bearing attribute in the
embedded ecosystem, because it's the mechanism that lets one HAL crate's
source tree serve dozens or hundreds of physically different
microcontrollers. A chip-family HAL like `stm32f4xx-hal` or `nrf-hal`
ships with one non-exclusive Cargo feature per supported part number
(`stm32f401`, `stm32f411`, `stm32f429`, ...), and nearly every
register-access module, interrupt vector entry, and peripheral count in
the crate is wrapped in `#[cfg(feature = "stm32f411")]` or an `any(...)`
combinator naming the handful of parts that share a given peripheral
layout. Only the module matching whichever single feature the
application actually enabled ever reaches the compiler — the rest are
discarded before type-checking, exactly as they'd need to be, since a
`GPIOF` peripheral module compiled for a chip that doesn't physically
have a GPIOF port wouldn't even type-check against that chip's memory
map.

A second common condition, `target_os = "none"`, is how firmware code
distinguishes a bare-metal build from a hosted one — embedded targets
(`thumbv7em-none-eabihf` and similar) report `target_os` as `"none"`
rather than `"linux"`/`"windows"`/`"macos"`, which is what lets a
`#![no_std]` crate that also wants to run part of its test suite on the
host gate its bare-metal-only code (a panic handler, a
`#[global_allocator]`, direct register access) behind
`#[cfg(target_os = "none")]`, while sharing everything else between both
builds.

Combinators matter just as much here as anywhere: a peripheral shared
across most of a chip family but absent from one variant is commonly
gated `#[cfg(any(feature = "stm32f401", feature = "stm32f411", feature = "stm32f429"))]`,
and mutually exclusive chip-selection features are enforced with a
`compile_error!` inside `#[cfg(not(any(...)))]` so selecting zero (or
more than one) chip feature fails loudly at compile time rather than
silently picking a default.

## Usage examples (Embedded)

### Selecting one HAL module per target chip feature

```
#[cfg(feature = "stm32f411")] // <- compiled only when the stm32f411 Cargo feature is enabled
mod stm32f411 {
    pub const GPIOA_BASE: u32 = 0x4002_0000;
}

#[cfg(feature = "stm32f429")] // <- compiled only when the stm32f429 Cargo feature is enabled
mod stm32f429 {
    pub const GPIOA_BASE: u32 = 0x4002_0000;
    pub const GPIOF_BASE: u32 = 0x4002_1400; // <- stm32f429 has a GPIOF port; stm32f411 doesn't
}

#[cfg(feature = "stm32f411")]
use stm32f411::GPIOA_BASE;

#[cfg(feature = "stm32f429")]
use stm32f429::GPIOA_BASE;
```

### Detecting a bare-metal build to install a panic handler

```
#[cfg(target_os = "none")] // <- true only on a bare-metal target, never when running `cargo test` on the host
mod bare_metal {
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }
}
```

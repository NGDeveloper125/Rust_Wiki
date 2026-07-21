---
title: "#[no_main]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: ["FFI (foreign function interface)", "Unsafe Rust"]
related_syntax: ["#![no_std]", "#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"]
see_also: ["#![no_std]"]
---

## Explanation

`#![no_std]` is an inner attribute placed at the very top of a crate root
that disables `#![no_main]`'s counterpart, ordinary Rust program startup:
it stops the compiler from generating the small runtime shim that
normally sets up the process, installs a panic/stack-overflow guard, and
then calls a Rust `fn main`.

Every ordinary Rust binary has more going on at startup than just the
`fn main` a programmer writes: the compiler emits a real, C-ABI `main`
entry point (or, on some platforms, a different startup symbol) that
performs setup — initializing thread-locals, installing signal/stack
guards, capturing `argc`/`argv` — before calling the programmer's `fn
main`. `#[no_main]` tells the compiler not to generate that startup
shim at all. It does **not** remove the requirement that *something* be
the program's actual entry point — it just means the crate itself
supplies that entry point some other way, typically via `#[no_mangle]`
on a function whose exact name and signature the platform's own loader or
a custom runtime crate expects (`_start` on many bare-metal targets,
`WinMain` for a Windows GUI subsystem entry, or a symbol name a custom
bootloader looks for).

`#[no_main]` is most often seen paired with `#![no_std]`: a `#![no_std]`
binary usually has no OS-provided C runtime to call an ordinary `main`
symbol at all, so the crate defines its own entry point directly (often
via a `#[entry]`-style attribute provided by a hardware-support crate,
which itself typically expands into a `#[no_mangle]` function under the
hood). It is used less often, but still meaningfully, in hosted contexts
where a custom C runtime or unusual embedding scenario needs to fully
control the process's true entry point.

## Basic usage example

```
#![no_main] // <- suppresses generation of the ordinary Rust startup shim / `main` call

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // this function itself is now the program's real entry point
    0
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A `#![no_std]` firmware crate has no operating system to call a `main`
symbol at all — the microcontroller's reset handler jumps directly to an
address the linker script names, so the crate supplies that exact symbol
itself instead of an ordinary `fn main`.

```
#![no_std]
#![no_main] // <- no OS-provided runtime exists to call `main`; the entry point is supplied below

use core::panic::PanicInfo;

#[unsafe(no_mangle)] // <- the linker script's reset vector points at this exact symbol name
pub extern "C" fn _start() -> ! {
    loop {
        // firmware main loop
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

**Why this way:** without `#[no_main]`, the compiler still expects to
generate a call into a conventional `main`, which requires a C runtime
this bare-metal target doesn't have; supplying `_start` directly under
`#[no_mangle]` is the standard shape embedded HAL crates and the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/index.html)
document for a custom entry point.

## Embedded Rust Notes

**Full support** — `#[no_main]` is overwhelmingly an embedded/OS-dev
attribute. Most embedded projects don't write it directly; a hardware
support crate's `#[entry]` macro (from crates like `cortex-m-rt`) expands
to the equivalent of `#![no_main]` plus a properly named, `#[no_mangle]`
reset-handler function, so application code just writes `#[entry] fn
main() -> !`. See [#![no_std]](no-std-attribute.md) for the broader
context `#[no_main]` almost always appears alongside.

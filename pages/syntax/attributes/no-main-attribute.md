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

## Usage examples

### Supplying a custom C-ABI entry point

```
#![no_main] // <- suppresses generation of the ordinary Rust startup shim / `main` call

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // this function itself is now the program's real entry point
    0
}
```

### Crossing an FFI boundary

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

Without `#[no_main]`, the compiler still expects to
generate a call into a conventional `main`, which requires a C runtime
this bare-metal target doesn't have; supplying `_start` directly under
`#[no_mangle]` is the standard shape embedded HAL crates and the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/index.html)
document for a custom entry point.

## Explanation (Embedded)

On a microcontroller, program "startup" isn't a call from a C runtime
into `main` at all — it's the CPU's hardware reset behavior. The very
first entry of the interrupt/exception vector table (a fixed array of
function-pointer-sized words placed at the start of flash by the linker
script) holds the initial stack pointer, and the second holds the address
of the **reset handler**: the hardware loads the program counter straight
from that table entry on power-up or reset, with no OS, no C runtime, and
no call instruction involved anywhere. `#![no_main]` is what tells the
compiler not to generate the ordinary startup shim that would otherwise
expect to call a conventional `fn main`, since there is nothing here for
that shim to be called by in the first place.

In practice, almost no embedded project writes the reset handler or
`#![no_main]`'s companion entry point by hand. `cortex-m-rt`'s `#[entry]`
attribute is the standard way in: it expands to roughly a `#[no_mangle]`
`extern "C" fn main` that the crate's own reset handler calls after
zeroing `.bss` and copying `.data` from flash into RAM, and it enforces at
compile time that only one `#[entry]` function exists and that it never
returns (`-> !`) — the same one-per-binary discipline
[`#[panic_handler]`](panic-handler-attribute.md) enforces, for the same
underlying reason: there is exactly one hardware reset vector to point
at. Concurrency frameworks built on top, like `RTIC`, also require
`#![no_main]` in the application crate, replacing the ordinary `fn main`
with their own `#[app]` macro that generates the hardware interrupt
handlers, shared-state locking, and task dispatch directly instead of a
single linear entry point.

## Usage examples (Embedded)

### Using cortex-m-rt's #[entry] instead of a hand-rolled reset handler

```
#![no_std]
#![no_main] // <- no C runtime exists to call an ordinary `main`

use panic_halt as _;
use cortex_m_rt::entry; // <- provides the real reset-vector wiring

#[entry] // <- expands to the equivalent of a #[no_mangle] reset-called entry function
fn main() -> ! {
    loop {
        // application logic
    }
}
```

### Building an interrupt-driven application with RTIC

```
#![no_std]
#![no_main] // <- RTIC's #[app] macro generates the real entry point and vector table wiring

use panic_halt as _;

#[rtic::app(device = nrf52840_hal::pac)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }

    #[task(binds = TIMER0)]
    fn on_timer(_cx: on_timer::Context) {
        // runs whenever the TIMER0 interrupt fires
    }
}
```

`#![no_main]` here isn't optional scaffolding — the `#[app]` macro's
generated code *is* the crate's entry point and interrupt table, so the
ordinary Rust startup shim `#![no_main]` suppresses would only be in the
way. See [`#![no_std]`](no-std-attribute.md) for the broader picture this
entry point sits inside.

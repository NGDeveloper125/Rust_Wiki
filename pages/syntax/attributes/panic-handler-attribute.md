---
title: "#[panic_handler]"
kind: attribute
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["Unsafe Rust"]
related_syntax: ["#![no_std]", "#[global_allocator]"]
see_also: ["#![no_std]", "#[global_allocator]"]
---

## Explanation

`#[panic_handler]` marks a function as the crate's panic handler — the
code that runs whenever `panic!` (directly or through something built on
it, like an out-of-bounds index or an `unwrap()` on `None`) is triggered
anywhere in the program. It is **required, exactly once, in every binary
that doesn't link `std`** — a `#![no_std]` binary crate will not compile
without one, because `std` is what normally supplies the default panic
handler (the one that prints a message, optionally unwinds the stack,
and aborts the process), and none of that machinery exists without an
underlying OS to unwind against or a stream to print to.

The function must have the signature `fn(&core::panic::PanicInfo) -> !`
— it receives a `PanicInfo` describing the panic (its message, and the
source location if that information wasn't stripped for size), and it
must never return, since there is no defined way to resume normal
execution after a panic in a `#![no_std]` context. What it actually does
is entirely up to the crate: the two overwhelmingly common shapes are
looping forever (`loop {}`, halting the processor in a known state for a
debugger to inspect) and triggering a hardware or software reset to
restart the device from a clean state. Some embedded projects instead log
the panic over a serial port or blink an LED pattern before halting or
resetting, if they have a cheap way to signal failure to whoever is
watching the hardware.

Only **one** `#[panic_handler]` function may exist in a dependency
graph's final linked binary — if two crates in the same binary each try
to define one (a common trap when two different "panic handler" crates
from crates.io both get pulled in transitively), the link fails. This is
why most real embedded projects pull in exactly one panic-handling crate
(`panic-halt`, `panic-reset`, `panic-probe`, and similar) rather than
writing `#[panic_handler]` by hand, picking the one whose failure
behavior (halt vs. reset vs. log-then-halt) fits the project.

In a `std`-linked binary, `#[panic_handler]` is not just unnecessary —
it's simply unavailable to use in the normal way, since `std` already
provides the process's one panic handler and installing another isn't
what this attribute is for; `std` code that wants to customize panic
*behavior* uses `std::panic::set_hook` instead, a completely different,
runtime-configurable mechanism built on top of the standard panic
handler, not a replacement for it.

## Basic usage example

```
#![no_std]

use core::panic::PanicInfo;

#[panic_handler] // <- required: the one function called whenever this no_std binary panics
fn panic(_info: &PanicInfo) -> ! {
    loop {} // halt here forever; a debugger can inspect state at this point
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A `#![no_std]` firmware binary needs a panic handler before it links at
all — the minimal, universally-applicable choice is to loop forever,
leaving the processor halted in a known, debuggable state rather than
attempting anything more elaborate that could itself fail.

```
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler] // <- the only panic handler allowed in the final linked binary
fn panic(info: &PanicInfo) -> ! {
    // In a real project a HAL crate would log `info` over a debug UART
    // or blink an LED pattern before halting; a bare minimal handler
    // simply stops execution in a known state.
    let _ = info;
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}
```

**Why this way:** `#![no_std]` removes `std`'s built-in panic handler
along with everything else `std` provides, so the crate must supply its
own or fail to link — the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/panicking.html)
documents "loop forever" as the simplest correct handler, and recommends
reaching for a published crate like `panic-halt` in real projects instead
of hand-writing this repeatedly, so the failure behavior is a deliberate,
reviewed choice rather than an ad hoc one per crate.

### Scenario: Handling and propagating errors

A project wants a panic to actually restart the device rather than sit
halted — the panic handler triggers a hardware reset instead of looping,
so a transient fault recovers on its own instead of requiring a manual
power cycle.

```
#![no_std]

use core::panic::PanicInfo;

#[panic_handler] // <- recovery strategy lives entirely inside this one function
fn panic(_info: &PanicInfo) -> ! {
    // stand-in for a hardware-specific reset, e.g. cortex_m::peripheral::SCB::sys_reset()
    reset_device();
}

fn reset_device() -> ! {
    loop {} // placeholder: a real target calls its reset intrinsic here instead
}
```

**Why this way:** because exactly one `#[panic_handler]` exists for the
whole binary, the crate's chosen recovery policy — halt for
debuggability during development, reset for resilience in a deployed
device — is a single, crate-wide decision; the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/panicking.html)
notes swapping between a halting and a resetting handler (e.g.
`panic-halt` vs. `panic-reset`) as a normal choice to make per build
profile rather than per panic site.

## Embedded Rust Notes

**Full support** — `#[panic_handler]` exists specifically for
`#![no_std]` contexts and is one of the first things any `#![no_std]`
binary crate must supply; see [#![no_std]](no-std-attribute.md) for the
broader picture of what else is lost and gained by opting out of `std`.
It has no equivalent role in hosted `std` binaries, where `std` already
installs the default panic handler and `std::panic::set_hook` is the
mechanism for customizing behavior instead.

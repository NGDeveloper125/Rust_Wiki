---
title: "#[panic_handler]"
kind: attribute
embedded_support: full
groups: ["No-std & Embedded Runtime", "Memory & Unsafe"]
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

## Usage examples

### Defining a minimal panic handler in a #![no_std] binary

```
#![no_std]

use core::panic::PanicInfo;

#[panic_handler] // <- required: the one function called whenever this no_std binary panics
fn panic(_info: &PanicInfo) -> ! {
    loop {} // halt here forever; a debugger can inspect state at this point
}
```

### Designing a public API

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

`#![no_std]` removes `std`'s built-in panic handler
along with everything else `std` provides, so the crate must supply its
own or fail to link — the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/panicking.html)
documents "loop forever" as the simplest correct handler, and recommends
reaching for a published crate like `panic-halt` in real projects instead
of hand-writing this repeatedly, so the failure behavior is a deliberate,
reviewed choice rather than an ad hoc one per crate.

### Handling and propagating errors

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

Because exactly one `#[panic_handler]` exists for the
whole binary, the crate's chosen recovery policy — halt for
debuggability during development, reset for resilience in a deployed
device — is a single, crate-wide decision; the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/start/panicking.html)
notes swapping between a halting and a resetting handler (e.g.
`panic-halt` vs. `panic-reset`) as a normal choice to make per build
profile rather than per panic site.

## Explanation (Embedded)

This page is about the attribute's own placement and linkage rules;
what actually happens once a panic fires — unwind versus abort, why most
firmware builds with `panic = "abort"`, and the specific handler crates
(`panic-halt`, `panic-itm`, `panic-probe`) built for that world — is
covered from the `panic!` side on the
[`panic!`](../macros/panic-macro.md) page, and isn't repeated here.

What belongs here is the contract the compiler and linker actually
enforce. The function must have exactly the signature
`fn(&core::panic::PanicInfo) -> !` — no other argument types, no other
return type, and never returning, since there's no defined way to resume
past a panic in a `#![no_std]` binary. And exactly **one** such function
may exist in a binary's final linked dependency graph: **zero** is a
link error (the linker has no symbol to satisfy the panic-handling
runtime hook every `#![no_std]` binary needs), and **two or more** is
also a link error (a duplicate-symbol failure), not silently resolved by
picking one. This second failure mode is the more common one in practice
— it happens when two different "panic handler" crates both end up in
the same dependency graph, each defining its own `#[panic_handler]`,
often because one dependency pulls in `panic-halt` while another pulls in
`panic-probe` transitively. The fix is always the same: make sure exactly
one panic-handler crate is an actual dependency of the final binary, not
a version or naming issue.

Because the handler crate is chosen purely by which one is a compiled-in
dependency — not by any explicit registration call — the idiomatic way to
supply one is importing it purely for its side effect:
`use panic_halt as _;` at the crate root. The `as _` discards the actual
import name (there's nothing to call), while still causing the crate's
`#[panic_handler]` function to be compiled into the binary and satisfy
the linker's requirement.

## Usage examples (Embedded)

### Selecting a handler crate purely for its side effect

```
#![no_std]
#![no_main]

use panic_probe as _; // <- imported only so its #[panic_handler] gets linked in
use defmt_rtt as _;   // <- transport panic-probe's formatted output rides over
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

### Keeping exactly one handler across build profiles

```
#![no_std]
#![no_main]

// Cargo.toml selects exactly one of these via mutually exclusive features,
// so exactly one #[panic_handler] is ever compiled into a given build:
//
//   [dependencies]
//   panic-halt  = { version = "0.2", optional = true }
//   panic-probe = { version = "0.3", optional = true, features = ["print-defmt"] }
//
//   [features]
//   release-halt = ["panic-halt"]
//   debug-probe  = ["panic-probe"]

#[cfg(feature = "release-halt")]
use panic_halt as _;

#[cfg(feature = "debug-probe")]
use panic_probe as _;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

Feature-gating the `use` this way makes the one-handler-per-binary rule
a build-time guarantee rather than something discovered as a duplicate-
symbol link error after two transitive dependencies each brought in their
own handler crate.

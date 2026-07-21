---
title: "#![no_std]"
kind: attribute
embedded_support: full
groups: ["No-std & Embedded Runtime", "Memory & Unsafe"]
related_concepts: ["Unsafe Rust", "FFI (foreign function interface)"]
related_syntax: ["#[panic_handler]", "#[global_allocator]", "#[no_main]"]
see_also: ["#[panic_handler]", "#[global_allocator]", "#[no_main]"]
---

## Explanation

`#![no_std]` is an inner attribute placed at the very top of a crate
root that opts the crate out of automatically linking the standard
library. Without it, every crate implicitly gets `extern crate std;` and
a standard prelude built on top of `std`; `#![no_std]` removes both,
leaving only `core` — the part of the standard library that has zero
dependency on an operating system, a heap allocator, or any platform-
specific runtime support — available by default.

**What `#![no_std]` means concretely:** `core` provides everything that
can be defined using only the CPU and memory in front of it — the
primitive types and their methods, `Option`, `Result`, iterators, slices,
`fmt`, atomics, and the panic/`unsafe` machinery — but nothing that
assumes an operating system underneath. This is why `#![no_std]` exists
at all: it lets Rust target bare-metal microcontrollers, kernel and
bootloader code, and any environment with no OS to provide threads,
files, sockets, or heap allocation.

**What's lost** going from `std` to `#![no_std]`, and why:

- **Anything needing an OS.** `std::thread` (no OS scheduler to create
  threads on), `std::fs` and `std::net` (no filesystem or network stack
  without an OS), `std::time::Instant`/`SystemTime` in their `std` form
  (no OS clock source assumed) — all gone. See the
  [Threads](../../concepts/concurrency-async/threads.md) concept page's
  Embedded Rust Notes for the "no OS, no `std::thread`" story in detail.
- **The heap, by default.** `Vec`, `Box`, `String`, `Rc`, and every other
  heap-allocating type are not in `core` at all — they live in `alloc`,
  a separate crate that sits between `core` and `std` in capability.
  `#![no_std]` code can opt back into these by adding
  `extern crate alloc;`, but only once the crate also supplies a
  `#[global_allocator]` — there's no default allocator without an OS to
  provide one. See the [Vec](../../concepts/collections-strings/vec.md)
  and [`#[global_allocator]`](global-allocator-attribute.md) pages for
  what this looks like in practice.
- **`std::collections`' non-`alloc` parts.** `HashMap`/`HashSet` also live
  in `alloc` (their hashing needs `alloc`-provided memory, though the
  default `RandomState` hasher itself actually depends on `std` for OS
  randomness — see the
  [HashMap and HashSet](../../concepts/collections-strings/hashmap-and-hashset.md)
  page's notes on that specific wrinkle) — usable with `alloc` wired up,
  but not from `core` alone.
- **The default panic/backtrace machinery.** `std`'s panic handler prints
  a message and a backtrace, optionally unwinds the stack, and aborts the
  process using OS facilities. None of that exists without `std`, so
  `#![no_std]` shifts the responsibility onto the crate itself: it must
  supply exactly one [`#[panic_handler]`](panic-handler-attribute.md)
  function, and stack unwinding is typically disabled entirely
  (`panic = "abort"`) since unwind tables need OS/target support most
  bare-metal targets don't have — see the
  [Panics and unwinding](../../concepts/error-handling/panic-and-unwinding.md)
  page's Embedded Rust Notes.
- **The normal program entry point**, in the common case where there's no
  OS to call an ordinary `main` at all — paired with
  [`#[no_main]`](no-main-attribute.md) and a custom, target-specific entry
  symbol instead.

**What's gained:** a `#![no_std]` binary makes zero assumptions about an
underlying operating system, which is exactly what makes it usable on a
microcontroller with a few kilobytes of RAM and no OS whatsoever, in a
kernel or bootloader (which *is* the thing that would otherwise provide
the OS), or in any other freestanding context. This is the foundational
attribute the entire embedded-Rust ecosystem is built on: hardware
abstraction layer crates, RTOS bindings, and async executors like
`embassy` all assume a `#![no_std]` base and layer their own
abstractions for concurrency, timing, and I/O on top of `core`/`alloc`
instead of `std`.

`#![no_std]` is a crate-level, all-or-nothing decision applied at the
crate root — it is not something toggled per-module. A workspace with
both a `#![no_std]` firmware crate and an ordinary `std` host-side tool
simply keeps them as two separate crates, sharing `#![no_std]`-compatible
logic through a third, shared `#![no_std]` library crate both depend on.

## Usage examples

### Opting a crate out of the standard library

```
#![no_std] // <- opts this crate out of linking std; only `core` is available by default

pub fn add(a: i32, b: i32) -> i32 {
    a + b // ordinary arithmetic needs nothing beyond core
}
```

### Designing a public API

A firmware crate for a microcontroller needs the minimal skeleton every
`#![no_std]` binary starts from: no heap, a custom entry point, and a
panic handler, since none of `std`'s defaults for any of these exist.

```
#![no_std] // <- no OS assumed: only core is available
#![no_main] // <- no OS means no conventional `main` symbol to call into

use core::panic::PanicInfo;

#[panic_handler] // <- required: replaces std's default panic/backtrace machinery
fn panic(_info: &PanicInfo) -> ! {
    loop {} // halt in a known state for a debugger to inspect
}

#[unsafe(no_mangle)] // <- the linker script's reset vector expects this exact symbol
pub extern "C" fn _start() -> ! {
    let result = add(2, 2);
    let _ = result;
    loop {}
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b // ordinary core-only logic works exactly as it does under std
}
```

This is the minimal shape every `#![no_std]` binary
needs — a panic handler and (absent an OS-provided one) a custom entry
point — before any application logic can be added; the
[embedded Rust book's minimal example](https://doc.rust-lang.org/stable/embedded-book/start/index.html)
builds up from exactly this skeleton, and in practice most real firmware
gets `#[no_main]`'s entry point and the panic handler from a hardware
support crate (`cortex-m-rt`, `panic-halt`) rather than writing them out
by hand as shown here for clarity.

### Handling and propagating errors

Once a `#![no_std]` crate needs heap-allocated collections — say, a
buffer of recent sensor readings — it opts into `alloc` explicitly and
must supply a `#[global_allocator]`, since neither exists by default
outside `std`.

```
#![no_std]
extern crate alloc; // <- opts back into Vec/Box/String; still not std

use alloc::vec::Vec;

pub fn recent_readings(history: &mut Vec<i32>, new_reading: i32) {
    if history.len() >= 16 {
        history.remove(0);
    }
    history.push(new_reading); // <- ordinary Vec, now available thanks to `alloc`
}

// A #[global_allocator] static (a bump allocator or a crate like
// embedded-alloc) must also be defined somewhere in this binary — see
// the #[global_allocator] page for a complete example.
```

`alloc` is a deliberately separate tier between `core`
and `std` precisely so a `#![no_std]` crate can choose "no heap at all"
or "heap available, but I still supply my own allocator and have no OS
underneath it" — the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/collections/index.html)
documents this as the standard path to using `Vec`/`String` in firmware,
distinct from a full `std` dependency.

## Embedded Rust Notes

**Full support** — trivially, since `#![no_std]` *is* the embedded-Rust
attribute; the toggle this page's own frontmatter drives is what every
other page's Embedded Rust Notes block is answering relative to. Rather
than restate each affected concept's story here, see: the
[Threads](../../concepts/concurrency-async/threads.md) page (no OS, no
`std::thread`), [Vec](../../concepts/collections-strings/vec.md) and
[HashMap and HashSet](../../concepts/collections-strings/hashmap-and-hashset.md)
(both need `alloc` plus a configured
[`#[global_allocator]`](global-allocator-attribute.md)), and
[Panics and unwinding](../../concepts/error-handling/panic-and-unwinding.md)
(a mandatory [`#[panic_handler]`](panic-handler-attribute.md), and
typically `panic = "abort"`) — each covers the specific delta in depth
without duplicating it here.

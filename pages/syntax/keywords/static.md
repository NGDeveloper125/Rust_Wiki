---
title: "static"
kind: keyword
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["Unsafe Rust"]
related_syntax: [const, unsafe, "&raw const / &raw mut"]
see_also: [const, unsafe]
---

## Explanation

`static NAME: T = value;` declares an item with a **fixed memory address
for the entire program** — one, single piece of storage, allocated once,
that every reference to `NAME` points at. This is the precise distinction
from [`const`](const.md), and it's worth being exact about: a `const` has
no memory address of its own at all — the compiler is free to copy its
value inline at every place it's used, the same way it would inline a
literal, so two different uses of the same `const` may not even share an
address if the compiler decides to materialize it twice. A `static`
never gets this treatment; `&NAME` yields the same address no matter
where in the program it's taken, which is exactly what makes a `static`
suitable as a genuine piece of global storage rather than a named
constant value.

A plain `static` is implicitly immutable — the same default every other
Rust binding has, and `static mut` is a separate, distinct form (below),
not a modifier you can add later. Note that "immutable" here describes
what safe code is allowed to do with the binding, not some inherent
property of the bytes in memory: the storage a `static` occupies is
ordinary writable memory unless the linker places it in a read-only
section, and unsafe code holding a raw pointer to it could still write
through that pointer. The compiler treats a plain `static` as read-only
from safe code because doing otherwise would break the aliasing
guarantees safe Rust depends on — a `static` is implicitly `'static` and
reachable from anywhere, including multiple threads at once, so if safe
code could write to it directly with no synchronization, that would be an
unsynchronized shared mutation: a data race, which safe Rust promises can
never happen. A shared plain `static` is effectively a global `&'static
T`; anything that needs to actually change over the program's lifetime
belongs in a `static` wrapping a synchronization primitive (`Mutex`,
`RwLock`, an atomic type) instead.

`static mut` is the escape hatch when a global truly needs direct
mutation: `static mut COUNTER: u32 = 0;` compiles, but every read or write
of `COUNTER` requires an [`unsafe`](unsafe.md) block, because the compiler
has no way to verify that two threads (or an interrupt handler and a main
loop) won't touch it at the same time with no synchronization — exactly
the data race plain `static` is designed to rule out. In practice, recent
Rust has been tightening this further: forming an ordinary `&`/`&mut`
reference directly to a `static mut` item is increasingly discouraged and
flagged by lint, since a reference carries validity and aliasing
guarantees a shared mutable global can't honestly promise for as long as
that reference exists. The idiomatic replacement is to obtain a raw
pointer with [`&raw const`/`&raw mut`](../operators/raw-borrow.md) instead
of a reference, and dereference through the pointer only inside the
`unsafe` block that needs it — narrowing the window where the aliasing
promise has to hold. Prefer a `static` `Mutex`/atomic over `static mut`
whenever the data is genuinely touched from more than one thread; reach
for `static mut` only in contexts (like a single-threaded interrupt
handler) where the synchronization story is guaranteed by something
outside the type system itself.

## Usage examples

### Declaring a fixed-address global constant

```
static MAX_CONNECTIONS: u32 = 100; // <- `static`: one fixed address for the whole program
```

### Bit manipulation and flags

A CRC-8 checksum routine used throughout a protocol driver needs the same
256-entry lookup table available everywhere it's called; computing that
table as a `const` would inline all 256 entries at every call site instead
of sharing one copy.

```
const fn build_crc_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut value = byte as u8;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 0x80 != 0 { (value << 1) ^ 0x07 } else { value << 1 };
            bit += 1;
        }
        table[byte] = value;
        byte += 1;
    }
    table
}

static CRC_TABLE: [u8; 256] = build_crc_table(); // <- `static`: one 256-byte table, shared by every caller

fn checksum(frame: &[u8]) -> u8 {
    frame.iter().fold(0u8, |crc, &b| CRC_TABLE[(crc ^ b) as usize])
}
```

A `const` of this size would be materialized wherever
it's referenced, bloating the binary by another 256 bytes per use site,
while a `static` places the table once and every call to `checksum`
indexes the same shared storage — the
[Reference's static items](https://doc.rust-lang.org/reference/items/static-items.html)
page documents exactly this fixed-single-address behavior as the
distinguishing property `const` doesn't have.

### Multi-threading

A server tracks the number of requests handled across worker threads; a
`static` atomic gives every thread the same shared counter with no lock
and no unsafe code, in contrast to what a `static mut` would require for
the same job.

```
use std::sync::atomic::{AtomicUsize, Ordering};

static REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0); // <- `static`: one shared counter, safe to touch from any thread

fn handle_request() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed); // <- no `unsafe`: the atomic type supplies its own synchronization
}

// AVOID:
// static mut REQUEST_COUNT_RAW: usize = 0;
// unsafe { REQUEST_COUNT_RAW += 1; } // <- data race if two threads reach this at once; compiler can't stop it
```

`AtomicUsize` gives the `static` interior mutability
with built-in synchronization, so ordinary safe code can increment it from
any thread — the [Book's shared-state concurrency
chapter](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
recommends exactly this shape over a raw `static mut`, which offers the
same global reach with none of the safety.

## Explanation (Embedded)

`static` means exactly what the classic Explanation above describes
under `#![no_std]` too — one fixed-address, program-long-lived piece of
storage — and that fixed address is precisely why it, rather than
[`const`](const.md), is the tool embedded code reaches for whenever more
than one part of the program needs to touch the *same* piece of storage:
a peripheral register block modeled as a singleton, or state genuinely
shared between an interrupt handler and the main loop. (This page is
about the `static` *item keyword* itself; the closely related `'static`
*lifetime* — which every `static` item's type is bound by, and which
shows up constantly on its own even where no `static` item is involved,
e.g. `thread::spawn`'s bound or a `'static` trait object — has its own
dedicated page, [`'static`](../lifetimes/static-lifetime.md), which
covers several embedded-specific scenarios — a `critical-section`-guarded
counter, a `#[global_allocator]`, a `heapless::spsc` queue split — in
depth. Those aren't repeated here.)

Two mechanisms specific to the *item* keyword are worth calling out.
First, a peripheral-access crate (hand-written, or `svd2rust`-generated
from a chip's SVD file) models each hardware peripheral as effectively a
`static` singleton, reached through a `take()`-style API
(`pac::Peripherals::take()`) that hands out the one instance exactly
once at runtime — this exists because there is exactly one physical
UART/GPIO/timer controller for the whole program to share, and modeling
that as a `static`-backed singleton, rather than a value freely
constructible anywhere, is what stops two independent parts of the
firmware from each believing they have exclusive access to the same
hardware. Second, `static mut` remains the direct way to give an
interrupt handler and `main` a shared, mutable global when no
synchronization wrapper is in the picture — every access requires
`unsafe`, and, per increasingly-recommended, lint-enforced practice,
forming an ordinary `&`/`&mut` reference to it directly is discouraged
in favor of a raw pointer obtained with `&raw const`/`&raw mut`,
narrowing the window where the aliasing promise has to hold (see
[`mut`](mut.md) for that pattern from the mutability-marker angle). In
new code, a `static` wrapping a `critical-section`-guarded
`Cell`/`RefCell`, or an atomic, is generally preferred over `static mut`
for exactly this reason — it gets the same interrupt/main-loop sharing
without ever needing an `unsafe` block at the access site.

## Usage examples (Embedded)

### Modeling a peripheral register block as a `static`-backed singleton

```
use stm32f4xx_hal::pac;

fn main() -> ! {
    let device = pac::Peripherals::take().unwrap(); // <- hands out the one `static`-backed peripheral instance
    let gpioa = &device.GPIOA;
    gpioa.odr.write(|w| w.odr5().set_bit());
    loop {}
}
```

### `static mut` as a direct interrupt/main-loop shared counter

```
static mut OVERFLOW_COUNT: u32 = 0; // <- `static mut`: one fixed address, reachable from both `main` and the ISR

#[interrupt]
fn TIM2() {
    unsafe {
        OVERFLOW_COUNT += 1; // <- every access needs `unsafe`: no compiler-checked synchronization here
    }
}

fn overflow_count() -> u32 {
    unsafe { OVERFLOW_COUNT }
}
```

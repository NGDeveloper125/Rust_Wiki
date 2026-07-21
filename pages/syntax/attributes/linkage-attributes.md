---
title: "#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: ["FFI (foreign function interface)", "Unsafe Rust", "Memory layout & repr"]
related_syntax: [extern, unsafe, static]
see_also: ["FFI (foreign function interface)"]
---

## Explanation

This page groups the small family of attributes that control how symbols
are named, located, and placed at **link time** — the step after
compilation where the linker stitches together object files, static
libraries, and system libraries into a final binary. None of these change
what a function or static *does*; they only change how the linker sees
it, which is why they cluster together despite touching different parts
of an `extern` boundary.

`#[unsafe(no_mangle)]` (edition 2024; plain `#[no_mangle]` on earlier
editions) is placed on a `fn` or `static` item and keeps that item's exact
Rust name as its linker symbol, instead of letting the compiler apply its
usual name-mangling scheme (which encodes the crate, module path, and
generic parameters into the symbol so multiple crates can define items
with the same plain name without colliding). A mangled name is
unpredictable across compiler versions and unusable from outside Rust, so
any function meant to be called *from* another language needs
`#[no_mangle]` to have a stable, guessable symbol. It requires wrapping in
`unsafe(...)` starting in edition 2024 because an incorrectly chosen name
can collide with another symbol in the final binary — a hazard the
compiler cannot check for.

`#[link(name = "...")]` is placed on an `unsafe extern` block and tells
the compiler which native library the linker should search for the
symbols declared inside that block — `#[link(name = "sqlite3")]` above an
`extern "C"` block passes `-lsqlite3` to the linker. A `kind` argument
(`kind = "static"`, `"dylib"`, `"framework"`) selects how that library
should be linked; `dylib` is the default.

`#[link_name = "..."]` is placed on an individual item **inside** an
`extern` block, and renames only what symbol that one item resolves to —
used when the Rust-side identifier can't or shouldn't match the foreign
symbol's actual name (a reserved Rust keyword, a name Rust's identifier
rules disallow, or simply a clearer Rust-side name than the C library
exports).

`#[export_name = "..."]` is closely related to `#[no_mangle]` but more
precise: instead of keeping the item's own Rust name unmangled,
`#[export_name = "raw_sensor_read"]` exports it under a name of your
choosing, which may differ entirely from the Rust identifier. `#[no_mangle]`
is effectively `#[export_name = "<the item's own name>"]`.

`#[link_section = "..."]` places a function or static into a specific
named section of the compiled binary (`.vector_table`, `.boot`, and
similar target-specific names), rather than letting the compiler and
linker choose an ordinary code or data section. This is how embedded
firmware places an interrupt vector table or a boot-stage routine at the
fixed address a linker script expects.

Two further attributes exist for narrower needs. `#[link_ordinal(N)]`
imports a symbol from a Windows DLL by its numeric export ordinal instead
of by name — needed only for DLLs that export by ordinal rather than by
name (some legacy Windows system libraries). `#[no_link]` marks an
`extern crate` declaration as contributing only its items to name
resolution without actually linking the crate's compiled output in —
a rare need, mostly internal to `rustc`'s own build.

## Usage examples

### Exporting a function under its exact, unmangled name

```
#[unsafe(no_mangle)] // <- keeps this exact name in the compiled binary's symbol table
pub extern "C" fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}
```

### Crossing an FFI boundary

A telemetry library exposes one function to C callers and links against a
vendor-supplied compression library whose header declares a function
under a name that isn't a valid Rust identifier style.

```
#[link(name = "vendor_codec")] // <- tells the linker to search for libvendor_codec
unsafe extern "C" {
    #[link_name = "vc_compress_v2"] // <- renames only this item's resolved symbol
    fn compress(input: *const u8, len: usize, out: *mut u8) -> i32;
}

#[unsafe(no_mangle)] // <- exports under the plain name `report_batch`, not a mangled one
pub extern "C" fn report_batch(data: *const u8, len: usize) -> i32 {
    unsafe {
        // SAFETY: `data` is a valid pointer to `len` initialized bytes,
        // guaranteed by this function's documented C-side contract.
        compress(data, len, std::ptr::null_mut())
    }
}
```

`#[link(name = "...")]` on the `extern` block and
`#[link_name]` on the individual item are the two separate knobs FFI code
needs — one names the library, the other renames a single symbol inside
it — while `#[unsafe(no_mangle)]` on the Rust-side export is required or
the compiler's mangled name would make `report_batch` unfindable by any
C caller; the [Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
and the [Rust Reference on linkage](https://doc.rust-lang.org/reference/linkage.html)
cover this combination as the standard shape for a two-way FFI boundary.

## Explanation (Embedded)

This whole attribute family is close to the load-bearing core of how a
`#![no_std]` binary comes together at all, not a peripheral feature of
it. `cortex-m-rt` finds an application's entry point and every interrupt
handler purely by symbol name: its `#[entry]` macro expands to
(approximately) a `#[unsafe(no_mangle)]` `extern "C" fn main`, and its
`#[interrupt]` macro expands each handler to a `#[unsafe(no_mangle)]`
function whose exact name must match one of the interrupt names the
runtime crate's own vector table already reserved a slot for (`USART1`,
`TIMER0`, `SPI0`, and so on, per the target's SVD-derived interrupt
list) — get the name wrong and the linker reports an undefined reference
to that vector-table slot, not a Rust-level type error. `#[unsafe(link_section = "...")]`
is what places that vector table itself — and other fixed-address
structures a boot ROM expects, like NXP's i.MX RT `FlexSPI` NOR boot
configuration block — at the exact flash offset the linker script or the
chip's boot ROM requires, rather than wherever the linker would otherwise
choose to put it.

`#[link(name = "...")]`/`#[link_name]` carry over unchanged from hosted
FFI, just against a different kind of library: instead of a general-
purpose system library, the "vendor library" in embedded FFI is typically
a chip maker's C SDK or HAL — ST's STM32Cube HAL, Nordic's `nrfx`, TI's
DriverLib — linked in as a prebuilt static library the Rust side calls
into through an `unsafe extern "C"` block, exactly as with any other C
FFI boundary; see
[FFI (foreign function interface)](../../concepts/memory-unsafe/ffi.md)
for that broader picture.

## Usage examples (Embedded)

### Naming a hardware interrupt handler for the vector table

```
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::{entry, interrupt};

#[entry]
fn main() -> ! {
    loop {}
}

#[interrupt] // <- expands to a #[unsafe(no_mangle)] fn named exactly "USART1"
fn USART1() {
    // this exact symbol name is what the vector table's USART1 slot expects;
    // a typo here becomes an undefined-reference link error, not a Rust error
}
```

### Linking directly against a vendor's C HAL

```
#![no_std]

#[link(name = "stm32f4xx_hal")] // <- tells the linker to search the prebuilt vendor HAL library
unsafe extern "C" {
    #[link_name = "HAL_GPIO_TogglePin"] // <- renames only this item's resolved C symbol
    fn hal_gpio_toggle_pin(port: *mut core::ffi::c_void, pin: u16);
}

#[unsafe(no_mangle)] // <- exported under a stable name a C-side test harness or bootloader can call
pub extern "C" fn toggle_status_led(gpio_port: *mut core::ffi::c_void) {
    unsafe {
        // SAFETY: `gpio_port` is a valid GPIO peripheral base address,
        // guaranteed by this function's documented C-side contract.
        hal_gpio_toggle_pin(gpio_port, 1u16 << 5);
    }
}
```

Both examples are the same two attributes doing the same jobs as in the
classic section — naming a symbol precisely, and telling the linker where
a foreign implementation lives — just against a vector table slot and a
chip vendor's HAL instead of a generic OS library.

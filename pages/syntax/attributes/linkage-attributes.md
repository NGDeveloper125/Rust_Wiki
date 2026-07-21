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

## Basic usage example

```
#[unsafe(no_mangle)] // <- keeps this exact name in the compiled binary's symbol table
pub extern "C" fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

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

**Why this way:** `#[link(name = "...")]` on the `extern` block and
`#[link_name]` on the individual item are the two separate knobs FFI code
needs — one names the library, the other renames a single symbol inside
it — while `#[unsafe(no_mangle)]` on the Rust-side export is required or
the compiler's mangled name would make `report_batch` unfindable by any
C caller; the [Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
and the [Rust Reference on linkage](https://doc.rust-lang.org/reference/linkage.html)
cover this combination as the standard shape for a two-way FFI boundary.

## Embedded Rust Notes

**Full support** — this whole family exists largely *for* embedded and
freestanding contexts. `#[link_section = "..."]` is the mechanism behind
placing an interrupt vector table or boot code at an address a linker
script expects, and `#[link(name = "...")]`/`#[link_name]` are exactly how
embedded HAL crates link against vendor C SDKs. See
[FFI (foreign function interface)](../../concepts/memory-unsafe/ffi.md)
for how these attributes fit into the broader FFI picture in `#![no_std]`
firmware.

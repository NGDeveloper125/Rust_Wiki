---
title: "extern"
kind: keyword
embedded_support: full
groups: ["Memory & Unsafe", "Modules, Crates & Visibility"]
related_concepts: ["FFI (foreign function interface)"]
related_syntax: ["extern crate", unsafe, "#[repr(...)]"]
see_also: ["extern crate", unsafe]
---

## Explanation

`extern` marks a boundary between Rust and a foreign calling convention —
almost always C, or something that presents a C-compatible ABI. It has
two main forms, plus an ABI-string vocabulary shared by both. (A third,
mostly-historical form, `extern crate`, declares a dependency on another
Rust crate rather than a foreign-language boundary and is covered on its
own page, [`extern crate`](extern-crate.md) — this page covers only the
FFI/ABI use of plain `extern`.)

**`extern "C" { ... }` — a block declaring foreign items.** This form
tells the compiler the signature of a function or `static` that lives in
a foreign library, without giving it a body — the compiler trusts the
declaration completely and generates calls against it. As of the 2024
edition, this block must itself be written `unsafe extern "C" { ... }`:
nothing about a foreign declaration can be checked by the compiler, so the
block is an explicit assertion that its contents are trusted. Under
earlier editions the same block was written without the leading `unsafe`;
edition 2024 code that omits it is a compile error, and `cargo fix`
handles the mechanical migration.

**`extern "C" fn` — a Rust function using the C calling convention.**
Applying `extern "C"` to a Rust function definition (rather than a block)
selects the C ABI for that function instead of Rust's own, unstable
internal calling convention, which is what makes the function callable
from foreign code at all. This is almost always paired with an attribute
that keeps the function's symbol name intact in the compiled binary — see
the linkage-attribute page for `#[unsafe(no_mangle)]` itself, which this
page does not re-explain.

**Other ABI strings.** `"C"` is by far the most common string after
`extern` and is also the default if the string is omitted entirely
(`extern fn` means `extern "C" fn`). `"system"` behaves like `"C"` on most
platforms but adapts to whatever convention the platform's native API
actually uses — notably `stdcall` for 32-bit Windows API calls — so code
targeting the Windows API should write `extern "system"` rather than
hard-coding `"C"`. `"Rust"` is the implicit calling convention every
ordinary Rust function already uses when no `extern` is written at all;
it has no stable, documented layout across compiler versions and exists
in the ABI-string vocabulary mainly for completeness and for function
pointer types that want to be explicit about it.

**The `safe` weak keyword, inside `unsafe extern` blocks.** Since the
whole `unsafe extern "C" { ... }` block is trusted wholesale, every item
declared inside it is, by default, unsafe to call or access — a caller
must still write `unsafe { ... }` at each use, exactly as with any other
`unsafe fn` or mutable `static`. The `safe` keyword, written before an
individual item's declaration inside the block (`safe fn tick_count() ->
u32;` or `safe static DEVICE_ID: u32;`), overrides this default for that
one item: it asserts the item is actually safe to call or read with no
surrounding `unsafe` block needed at the call site. This is deliberately
per-item rather than block-wide — a header full of foreign declarations
usually mixes genuinely pure, side-effect-free functions (reading a
hardware ID, returning a constant) with ones that mutate shared state or
have pointer-validity preconditions, and `safe` lets the binding's author
be selective about which is which instead of forcing every caller to
write `unsafe` even for the harmless ones. `safe` is a weak keyword: it
has no special meaning outside this exact position, so it remains usable
as an ordinary identifier elsewhere.

See [FFI](../../concepts/memory-unsafe/ffi.md) for why this boundary
exists at all and how it fits into a real crate's design.

## Basic usage example

```
unsafe extern "C" {
    fn abs(input: i32) -> i32; // <- `extern "C"`: declares a foreign function's signature
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A media-processing crate needs both directions at once: calling a vendor
compression library to shrink a buffer, and exposing a Rust callback that
the same library invokes to report progress as it works.

```
unsafe extern "C" {
    fn compress_buffer(data: *const u8, len: usize, out_len: *mut usize) -> *mut u8; // <- `extern "C"`: calling INTO the foreign library
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out_len: usize = 0;
    unsafe {
        // SAFETY: `input` is a valid slice for its full length; `out_len`
        // is a valid, aligned `usize` the library writes its result into.
        let ptr = compress_buffer(input.as_ptr(), input.len(), &mut out_len);
        let bytes = std::slice::from_raw_parts(ptr, out_len).to_vec();
        libc_free(ptr);
        bytes
    }
}

unsafe extern "C" {
    fn libc_free(ptr: *mut u8);
}

#[unsafe(no_mangle)] // <- keeps the symbol name stable so the C library can call it by name
pub extern "C" fn on_compress_progress(percent: u32) {
    // <- `extern "C"`: crossing FROM C back into Rust, called by the library itself
    eprintln!("compressing: {percent}%");
}
```

**Why this way:** `extern "C"` on both the declared foreign function and
the exported callback selects the same calling convention on each side of
the boundary, which is what lets the two languages agree on how arguments
and the return value are passed at all — the
[Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
covers calling out and being called back as the two symmetric halves of
one boundary.

### Scenario: Designing a public API

A binding crate around a legacy timer/watchdog HAL header wants to let
callers read the current tick count without writing `unsafe` themselves,
while still requiring `unsafe` for the one function that actually resets
hardware state.

```
unsafe extern "C" {
    safe fn hal_tick_count() -> u32; // <- `safe`: pure, side-effect-free — no `unsafe` needed to call it
    fn hal_watchdog_reset(); // <- no `safe`: mutates hardware state, callers must still write `unsafe`
}

pub fn uptime_ticks() -> u32 {
    hal_tick_count() // <- callable directly: `safe` opted this one item out of the block's default
}

pub fn feed_watchdog() {
    unsafe {
        // SAFETY: resetting the watchdog is safe to call from any
        // context; documented by the vendor as reentrant.
        hal_watchdog_reset(); // <- still requires `unsafe`: this item wasn't marked `safe`
    }
}
```

**Why this way:** marking only the genuinely side-effect-free declaration
`safe` lets a wrapper crate be precise about which parts of a trusted C
header are actually safe to expose directly, instead of either forcing
`unsafe` on every foreign call or (worse) wrapping the whole header in a
blanket safe API without actually checking each item — the per-item
`safe` keyword is exactly the edition-2024 mechanism the
[Rust Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)
introduced alongside `unsafe extern` blocks for this purpose.

## Embedded Rust Notes

**Full support.** `extern` is, if anything, more central in embedded Rust
than in hosted Rust: an interrupt vector table is a table of `extern "C"`
function pointers the linker wires up, and vendor-supplied peripheral
libraries are almost always bound through `unsafe extern "C"` blocks
rather than called through any higher-level mechanism. The main practical
difference from hosted FFI is linkage — a `#![no_std]` crate typically
pulls in the vendor's compiled library via a `build.rs` script rather than
the system linker finding a shared library on its default search path.

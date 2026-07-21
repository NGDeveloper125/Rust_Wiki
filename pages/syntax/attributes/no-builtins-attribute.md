---
title: "#[no_builtins]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: ["Unsafe Rust"]
related_syntax: ["#![no_std]"]
see_also: ["#![no_std]"]
---

## Explanation

`#![no_builtins]` is an inner attribute placed at the top of a crate root
that disables the compiler's freedom to silently substitute certain code
patterns with calls to builtin/intrinsic implementations — typically
memory-related routines like `memcpy`, `memset`, and `memmove` that LLVM
recognizes and can lower a plain loop or a `[T]`-copying operation into
automatically, on the assumption that a C-runtime-compatible
implementation of those symbols will be available at link time.

In ordinary hosted Rust, those symbols always exist — they come from the
platform's C runtime (`libc`) that's already linked in — so this
substitution is invisible and harmless; it's simply a codegen
optimization. In a freestanding or kernel context, where there is no C
runtime and no guarantee those symbols exist at all, that same
substitution becomes a linker error or, worse, a silent jump to whatever
symbol happens to occupy that name if one was defined incidentally.
`#![no_builtins]` tells the compiler not to perform this substitution, so
the crate's own code is the only thing that can introduce a dependency on
`memcpy`-like symbols — and if it does need them, it (or a `compiler_builtins`-style
crate) must provide its own implementations explicitly.

This is a narrow, kernel/freestanding-specific attribute: it matters only
to code that cannot assume *any* runtime support functions exist, which
in practice means operating-system kernels and similarly deep
bare-metal/freestanding crates — most `#![no_std]` embedded application
code sits on top of a runtime crate that already handles this and never
needs to reach for `#![no_builtins]` itself.

## Usage examples

### Disabling implicit libc-builtin substitution

```
#![no_std]
#![no_builtins] // <- forbids the compiler from silently assuming a libc-provided memcpy/memset exist
```

### Designing a public API

A minimal kernel crate provides its own low-level memory primitives
rather than linking against any C runtime — `#![no_builtins]` guarantees
the compiler never quietly assumes a `memcpy` symbol will show up from
somewhere else.

```
#![no_std]
#![no_builtins] // <- this kernel supplies its own memory routines; nothing may assume libc's exist

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            // SAFETY: caller guarantees `dest` and `src` are valid for `n`
            // bytes and don't overlap, per the standard memcpy contract.
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}
```

Without `#![no_builtins]`, the compiler may still lower
some Rust code into a call to `memcpy` even though this crate defines its
own — in an environment with no C runtime, an unexpected implicit call
to a symbol nothing provides is a link failure at best; the
[rustc book](https://doc.rust-lang.org/beta/unstable-book/language-features/no-core.html)
and kernel-development guides (e.g. the
[Writing an OS in Rust](https://os.phil-opp.com/freestanding-rust-binary/) series)
document `#![no_builtins]` as part of the freestanding-binary checklist
alongside `#![no_std]`.

## Explanation (Embedded)

Ordinary application-level `#![no_std]` firmware almost never reaches for
`#![no_builtins]`, and it's worth being direct about why: `rustc`
automatically links a crate called `compiler_builtins` into every
`#![no_std]` binary, which supplies `memcpy`/`memset`/`memmove` and, on
targets without hardware floating-point (many Cortex-M0 parts), the
software floating-point routines the compiler otherwise assumes libgcc
would provide. A typical HAL-based application — reading sensors, driving
a display, talking over a UART — depends on that implementation
transparently and has no reason to disable it.

`#![no_builtins]` earns its keep in a narrower layer beneath ordinary
application firmware: bootloaders and safety-critical/certified cores.
A first-stage bootloader that runs before the flash controller is fully
configured, or that copies its own next stage out of raw flash, sometimes
needs full control over exactly which `memcpy` implementation runs and
when, rather than trusting whichever one `compiler_builtins` happens to
lower a copy into — `#![no_builtins]` is the guarantee that no code path
introduces a call to `compiler_builtins`'s version behind the scenes. In
safety-qualified environments (DO-178C avionics code, ISO 26262
automotive code), a project may also need to substitute a *certified*
implementation of these routines for the standard, uncertified one that
ships with the compiler, which similarly requires disabling the implicit
substitution first.

## Usage examples (Embedded)

### A bootloader supplying its own memcpy instead of relying on compiler_builtins

```
#![no_std]
#![no_builtins] // <- this bootloader controls its own memory routines; nothing may assume the compiler's

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            // SAFETY: caller guarantees `dest`/`src` are valid for `n` bytes and don't overlap.
            *dest.add(i) = *src.add(i);
        }
        i += 1;
    }
    dest
}
```

### Ordinary firmware needs none of this

```
#![no_std]
#![no_main]
// No #![no_builtins] here: this application-level firmware relies on
// compiler_builtins' memcpy/memset (linked in automatically) exactly
// like the overwhelming majority of #![no_std] HAL-based projects do.

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let buffer = [0u8; 64];
    let mut copy = [0u8; 64];
    copy.copy_from_slice(&buffer); // compiles to a memcpy call, satisfied by compiler_builtins

    loop {}
}
```

The second example is the realistic default for nearly all embedded
Rust; `#![no_builtins]` belongs to the small slice of bootloader- and
certification-level code that must control this, not to application
firmware sitting on a HAL.

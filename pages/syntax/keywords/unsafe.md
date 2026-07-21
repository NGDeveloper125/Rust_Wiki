---
title: "unsafe"
kind: keyword
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["Unsafe Rust", "FFI (foreign function interface)", "The undefined-behavior boundary", "Raw pointers (*const T / *mut T)"]
related_syntax: [extern, union, "&raw const / &raw mut", "*"]
see_also: [extern, union]
---

## Explanation

`unsafe` appears in four distinct grammatical positions, each gating a
different capability. It is never a blanket "turn off checking" switch —
ordinary type checking, lifetimes, and most borrow rules still apply
everywhere `unsafe` shows up.

**`unsafe { ... }` — a block.** Inside this block, and only inside it,
five specific operations become legal that are compile errors anywhere
else: dereferencing a raw pointer (`*ptr`); calling an `unsafe fn` (or an
unsafe trait method, or a compiler intrinsic); accessing or mutating a
mutable `static`; implementing an unsafe trait (written as `unsafe impl
Trait for Type { ... }`, itself a variant of this same keyword rather than
a block); and reading or writing a field of a `union`. This is a precise,
closed list — the [Rustonomicon](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
enumerates exactly these five, nothing more. An `unsafe` block does not
disable bounds checks, integer overflow checks in debug builds, or the
borrow checker's handling of ordinary references; it only unlocks these
five operations.

**`unsafe fn` — a function whose caller carries a safety obligation.**
Marking a function `unsafe fn` means the compiler cannot verify some
precondition the function needs to behave correctly — a pointer must be
non-null, an index must be in range, a byte slice must be valid UTF-8 —
and it is the *caller's* job to check that precondition before calling.
The compiler enforces only the mechanical half of this contract: every
call to an `unsafe fn` must itself be wrapped in an `unsafe { ... }`
block, forcing the caller to explicitly opt in. It cannot enforce the
semantic half — that the caller actually upheld the precondition — which
is why an `unsafe fn`'s doc comment should always spell out its
requirements under a `# Safety` heading, and every call site should carry
a `// SAFETY:` comment explaining why the requirement holds there.

**`unsafe trait` — a trait whose invariant the compiler can't check.**
`unsafe trait Trait { ... }` declares that implementing `Trait` correctly
requires upholding some invariant beyond what the trait's method
signatures alone express — the canonical examples are `Send` (safe to
transfer ownership to another thread) and `Sync` (safe to share by
reference across threads), where nothing in the method list captures
"this type has no hidden aliasing that would race." Because the compiler
can't check that invariant, it requires the implementer to assert it
explicitly by writing `unsafe impl Trait for Type { ... }` instead of a
plain `impl` — the `unsafe` on the `impl` is the implementer's promise
that they checked the invariant by hand.

**`unsafe extern` and `unsafe(attr)` — edition 2024 additions.** As of the
2024 edition, an `extern` block declaring foreign items must itself be
written `unsafe extern "C" { ... }`, since the compiler has no way to
verify a foreign declaration matches its real signature; see
[`extern`](extern.md) for the full grammar of these blocks. The same
edition also requires wrapping certain attributes that affect global
linkage — most commonly `#[unsafe(no_mangle)]` — in `unsafe(...)`. Both
are brief extensions of the same "opt in explicitly to what the compiler
can't verify" idea and are covered in full on [`extern`](extern.md) and
the linkage-attribute page rather than repeated here.

See [Unsafe Rust](../../concepts/memory-unsafe/unsafe-rust.md) for the
full mental model (contract, not compiler bypass) and
[The undefined-behavior boundary](../../concepts/memory-unsafe/the-undefined-behavior-boundary.md)
for what goes wrong when the contract is violated.

## Basic usage example

```
let value: i32 = 10;
let ptr = &value as *const i32;
let read = unsafe { *ptr }; // <- `unsafe` block: required to dereference a raw pointer
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A firmware updater needs a CRC-32 checksum routine from a vendor-supplied
C library; the compiler has no way to check what that foreign function
actually does with the pointer and length it's given.

```
unsafe extern "C" {
    fn crc32(data: *const u8, len: usize) -> u32; // <- foreign function; body is opaque to the compiler
}

fn checksum(firmware_image: &[u8]) -> u32 {
    unsafe {
        // SAFETY: `firmware_image` is a valid Rust slice for its full
        // length, so `.as_ptr()`/`.len()` describe a real, in-bounds
        // buffer that `crc32` (documented as read-only, non-retaining)
        // can safely read.
        crc32(firmware_image.as_ptr(), firmware_image.len()) // <- `unsafe` block: calling into C
    }
}
```

**Why this way:** the compiler can verify the *signature* of `crc32` but
nothing about its body or side effects, so `unsafe` marks the exact call
site where a human is vouching for the contract instead — the
[Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
treats every call across this boundary as needing exactly this kind of
manual justification.

### Scenario: Designing a public API

An audio ring buffer wants to skip bounds checks on its hot read/write
path without ever letting a caller trigger an out-of-bounds access — the
`unsafe fn` doing the raw write documents its contract, and the only
caller is a safe method that has already checked it.

```
pub struct AudioRingBuffer {
    samples: Box<[f32]>,
    len: usize,
}

impl AudioRingBuffer {
    /// # Safety
    /// `index` must be less than `self.samples.len()`.
    unsafe fn write_unchecked(&mut self, index: usize, sample: f32) {
        unsafe {
            // SAFETY: upheld by this function's caller, per the doc above.
            *self.samples.get_unchecked_mut(index) = sample; // <- `unsafe` block: the five-operations dereference/skip-check
        }
    }

    pub fn push(&mut self, sample: f32) {
        if self.len < self.samples.len() {
            unsafe {
                // SAFETY: `self.len < self.samples.len()` was just checked.
                self.write_unchecked(self.len, sample); // <- `unsafe` block: calling an `unsafe fn`
            }
            self.len += 1;
        }
    }
}
```

**Why this way:** keeping `write_unchecked` private and calling it from
exactly one already-checked call site is the "contain unsafety in small
modules" idiom described on [Unsafe Rust](../../concepts/memory-unsafe/unsafe-rust.md) —
every other method on `AudioRingBuffer` stays ordinary safe code, and the
one invariant the module relies on (`len <= samples.len()`) is enforced
in a single place.

## Embedded Rust Notes

**Full support.** `unsafe` is core-language and, if anything, appears more
often in embedded code than hosted code: reading a memory-mapped
peripheral register through a raw pointer, writing an interrupt vector
table entry, and sharing a mutable `static` between an interrupt handler
and the main loop are all routine, and every one of them requires
`unsafe`. HAL crates concentrate these blocks in one low-level layer and
expose a safe, typed API (the `embedded-hal` traits) to the rest of the
firmware.

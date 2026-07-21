---
title: "The undefined-behavior boundary"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "FFI / Interop", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, extern, "*"]
see_also: ["Unsafe Rust", "Raw pointers (*const T / *mut T)", "FFI (foreign function interface)", "Memory layout & repr"]
---

## Explanation

Undefined behavior (UB) is what happens when code violates one of the
invariants the Rust compiler is allowed to assume without checking — and
"undefined" means literally that: the language specification places no
constraint at all on what happens next. This is different from a panic
(a controlled, defined abort) or a compile error (caught before the
program even runs). UB can produce a wrong answer, a crash, silent memory
corruption that surfaces somewhere else entirely, or — because the
optimizer is allowed to assume UB never happens — code that behaves
correctly in a debug build and incorrectly in release, or that behaves
differently after an unrelated change elsewhere in the file.

Safe Rust's entire value proposition rests on the promise that safe code
can never trigger UB, no matter what it does. That promise only holds
because every `unsafe` block is a place where the programmer takes over
responsibility for a specific invariant the compiler would otherwise have
enforced. The undefined-behavior boundary is exactly that line: everything
on the safe side is checked by the compiler; everything on the unsafe
side is checked by whoever wrote the `unsafe` block, and getting it wrong
doesn't just break that one function — it can invalidate assumptions the
optimizer made anywhere else in the program, since the compiler is
allowed to assume UB never occurs when it reasons about the rest of the
code.

The concrete rules are things like: never dereference a null, dangling,
or misaligned pointer; never create two `&mut` references (or a `&mut`
and a `&`) to the same memory at the same time; never produce a reference
that outlives the data it points to; never read uninitialized memory as
if it were initialized; never call a function with a `#[repr]` or
signature mismatch across an [FFI](ffi.md) boundary; and never violate a
data-race-freedom guarantee across threads. Each item on the list is
narrow and specific — this is what makes `unsafe` code auditable at all —
but the consequence of missing one is disproportionate to how small the
mistake looks in the source.

Because the boundary is enforced by discipline rather than the compiler,
the practical defense is process, not vigilance alone: keep `unsafe`
blocks small and few, document the exact invariant each one relies on
with a `// SAFETY:` comment, and expose only a safe API from any module
containing `unsafe` so the rest of the codebase can rely on the compiler
again past that module's boundary. Tools like Miri (an interpreter that
detects many UB patterns dynamically) exist precisely because "read the
code carefully" doesn't scale as the only line of defense.

## Basic usage example

```
let mut value = 42;
let ptr: *mut i32 = &mut value;

unsafe {
    // SAFETY: `ptr` was just derived from the live, uniquely-owned
    // reference `&mut value`, and nothing else accesses `value` while
    // this dereference happens.
    *ptr += 1; // <- staying on the defined side: a valid, non-aliased, aligned dereference
}
println!("{value}");
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

An FFI wrapper around a C function that returns a pointer must never let
that pointer become dangling or null on the Rust side — the contract has
to be checked and upheld manually, since C gives the compiler no help
here.

```
unsafe extern "C" {
    fn device_open(name: *const u8) -> *mut u8; // <- returns null on failure, per the vendor's header
    fn device_close(handle: *mut u8);
}

pub struct Device {
    handle: *mut u8,
}

impl Device {
    pub fn open(name: &std::ffi::CStr) -> Option<Self> {
        let handle = unsafe {
            // SAFETY: `name` is a valid, NUL-terminated CStr for the
            // duration of this call, satisfying device_open's contract.
            device_open(name.as_ptr() as *const u8)
        };
        if handle.is_null() {
            None // <- never hand out a Device wrapping a dangling/null pointer
        } else {
            Some(Device { handle })
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `self.handle` is non-null (checked in `open`) and
            // closed at most once, since Drop runs exactly once.
            device_close(self.handle);
        }
    }
}
```

**Why this way:** dereferencing or passing along a null or already-closed
handle is exactly the kind of UB the C side can't protect Rust from —
checking `is_null()` immediately after the FFI call, and freeing exactly
once in `Drop`, keeps the invariant "a `Device`'s handle is always valid"
true everywhere else in the program, which the
[Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html) treats as the
caller's responsibility at every FFI edge.

### Scenario: Designing a public API

Documenting the exact safety invariant an `unsafe fn` requires from its
caller is what lets the rest of the crate use it correctly without
re-deriving the reasoning each time — an undocumented `unsafe fn` shifts
the entire burden of avoiding UB onto every future caller.

```
/// # Safety
/// `ptr` must be non-null, properly aligned for `T`, and point to a
/// live, initialized `T` for the duration of the call. The caller must
/// ensure no other reference to that memory exists concurrently.
pub unsafe fn read_register<T: Copy>(ptr: *const T) -> T {
    unsafe {
        // SAFETY: upheld by this function's caller, per the doc comment above.
        *ptr // <- the invariant is documented at the fn boundary, not re-derived at each call site
    }
}
```

**Why this way:** an `unsafe fn` shifts responsibility for its invariant
onto the caller, so the invariant has to be written down, not left
implicit — Clippy's
[`undocumented_unsafe_blocks`](https://rust-lang.github.io/rust-clippy/master/index.html#undocumented_unsafe_blocks)
lint and the [Rustonomicon's meet-safe-and-unsafe
chapter](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
both treat a documented contract as the difference between an audited
unsafe function and a landmine.

## Embedded Rust Notes

**Full support.** The undefined-behavior rules are core-language and
identical whether or not `std` is available — arguably the stakes are
higher in embedded code, since there is often no OS memory protection to
turn a bad dereference into a clean segfault instead of silently
corrupting adjacent hardware state or another task's memory. Volatile
access to memory-mapped registers (`core::ptr::read_volatile`/
`write_volatile` instead of a plain dereference) is one embedded-specific
UB pitfall with no hosted-Rust equivalent: an ordinary read/write can be
reordered or elided by the optimizer in ways that are invisible to normal
memory but silently drop a required hardware side effect.

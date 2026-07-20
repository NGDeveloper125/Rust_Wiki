---
title: "Contain unsafety in small modules"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Encapsulation"]
related_syntax: [unsafe, mod]
see_also: ["Unsafe Rust", "Modules", "Visibility & privacy (pub and friends)"]
---

## Explanation

This idiom is about *where the boundary of trust lives* around unsafe
code, not about `unsafe` itself — see [Unsafe Rust](../memory-unsafe/unsafe-rust.md)
for the mechanism and what the keyword actually changes. The idea:
instead of scattering `unsafe` blocks across a codebase wherever a raw
pointer or FFI call happens to be convenient, gather everything that
depends on a given unsafe invariant into one small, deliberately scoped
[module](../modules-crates-visibility/modules.md), keep every field and
helper that participates in the invariant private to that module, and
expose only a safe API at the module's boundary. Every caller outside
the module then gets ordinary safe Rust back — the compiler re-enforces
all of its usual guarantees the moment code crosses back out — because
the module's author has already discharged the unsafe contract once, in
one reviewable place, instead of it being re-litigated at every call
site that happens to need it.

The size of the module matters as much as its existence. A module with
five unsafe operations spread across a thousand lines of unrelated safe
helper code is barely better than no containment at all — a reviewer
auditing "everywhere this invariant could be broken" still has to read
the whole file. A module of thirty focused lines wrapping one raw
pointer and nothing else is auditable in a single sitting, and — just as
importantly — small enough that every private field it touches can be
verified as reachable *only* through the module's own methods, since
[visibility](../modules-crates-visibility/visibility-and-privacy.md) is
what makes "only this module's code can violate this invariant" true in
the first place. Widening the module later — adding unrelated public
methods, letting other code obtain a raw pointer out of it — directly
widens the audit surface back out.

This is why the idiom pairs naturally with FFI wrappers and low-level
data structures: a foreign library's opaque handle, or a hand-rolled
allocator's raw pointer arithmetic, gets wrapped in a small module
exposing a safe struct, safe methods, and often a `Drop` implementation
that releases the resource — everything the unsafe code depends on
stays a private implementation detail the module alone is responsible
for. The same shape shows up constantly in embedded hardware-abstraction
crates: a peripheral's raw register access is unsafe, but the module
wraps it in a small, safe, typed API so application code above it never
writes `unsafe` at all.

## Basic usage example

```
mod counter {
    use std::cell::UnsafeCell;

    pub struct FastCounter { // <- the only public item; every unsafe detail stays inside this module
        value: UnsafeCell<u32>,
    }

    impl FastCounter {
        pub fn new() -> Self {
            Self { value: UnsafeCell::new(0) }
        }

        pub fn increment(&self) {
            unsafe {
                // SAFETY: called only through `&self` from single-threaded code in this example;
                // no other reference to `value` is live at the same time.
                *self.value.get() += 1;
            }
        }

        pub fn get(&self) -> u32 {
            unsafe { *self.value.get() }
        }
    }
}

let counter = counter::FastCounter::new();
counter.increment();
println!("{}", counter.get());
```

## Best practices & deeper information

### Scenario: Designing a public API

A small bump-allocator hands out slots from one pre-allocated buffer;
every pointer-arithmetic operation stays inside the module, and the
public API only ever returns safe references tied to the arena's own
lifetime.

```
mod arena {
    pub struct Arena {
        buffer: Vec<u8>,
        used: usize,
    }

    impl Arena {
        pub fn new(capacity: usize) -> Self {
            Self { buffer: vec![0; capacity], used: 0 }
        }

        pub fn alloc(&mut self, bytes: usize) -> &mut [u8] {
            assert!(self.used + bytes <= self.buffer.len(), "arena exhausted");
            let start = self.used;
            self.used += bytes;
            unsafe {
                // SAFETY: `start..start + bytes` was just checked to be within
                // `self.buffer`'s bounds, and no other slice into `buffer` is live.
                std::slice::from_raw_parts_mut(self.buffer.as_mut_ptr().add(start), bytes) // <- the module's one raw-pointer operation
            }
        }
    }
}

let mut arena = arena::Arena::new(64);
let slot = arena.alloc(8); // <- caller only ever sees a safe `&mut [u8]`, never a raw pointer
slot[0] = 42;
```

**Why this way:** the only place `Arena`'s bounds invariant could be
violated is inside this one module, which is exactly the small,
auditable "unsafe core with a safe API around it" shape the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
recommends — applied here to a bump allocator instead of a fixed
buffer.

### Scenario: Crossing an FFI boundary

Wrapping a C standard library function so the single `unsafe extern`
call and its safety justification live in one place, with the rest of
the crate only ever seeing a safe, callable type.

```
mod libm {
    unsafe extern "C" {
        fn abs(input: i32) -> i32;
    }

    pub struct SafeAbs; // <- the module's only public item

    impl SafeAbs {
        pub fn compute(&self, input: i32) -> i32 {
            unsafe {
                // SAFETY: `abs` from the C standard library is a pure function
                // with no preconditions on its i32 argument.
                abs(input) // <- the only place in the crate this raw FFI call is allowed to happen
            }
        }
    }
}

let calculator = libm::SafeAbs;
println!("{}", calculator.compute(-7));
```

**Why this way:** keeping the single point where the program vouches for
a foreign function's contract private to one module, rather than letting
`unsafe extern` calls appear anywhere a crate feels like reaching for
them, is exactly the containment the
[Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
assumes when it discusses trusting a foreign function's behavior.

## Embedded Rust Notes

**Full support.** This is a source-organization idiom, not a runtime
feature, so it costs nothing and applies identically under `#![no_std]`.
It is, if anything, used more heavily there: hardware abstraction layer
crates concentrate all of a driver's raw register access — inherently
`unsafe` — inside one small internal module, exposing a safe, typed API
(often built on `embedded-hal` traits) so application code above the
driver never needs to write `unsafe` itself, the same shape as
[Unsafe Rust](../memory-unsafe/unsafe-rust.md)'s embedded notes describe.

---
title: "#[global_allocator]"
kind: attribute
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: ["Unsafe Rust"]
related_syntax: ["#![no_std]", "#[panic_handler]"]
see_also: ["#![no_std]", "#[panic_handler]"]
---

## Explanation

`#[global_allocator]` marks a `static` item implementing the `GlobalAlloc`
trait as the allocator backing every heap allocation `alloc`'s types make
— `Vec`, `Box`, `String`, `Rc`, and everything else in `alloc` (and, by
extension, `std`, which re-exports them) call through whichever type is
marked `#[global_allocator]` whenever they need to allocate or free heap
memory.

In an ordinary hosted `std` program, a default global allocator (the
system allocator — `malloc`/`free` on most platforms) is wired in
automatically; `#[global_allocator]` is optional there, used only to
*replace* the default — the most common reason being to swap in a
faster or more instrumented allocator such as `jemalloc` or `mimalloc`
for a performance-sensitive application.

In a `#![no_std]` context that also uses `extern crate alloc;` to get
access to `Vec`, `Box`, and friends, `#[global_allocator]` stops being
optional and becomes **required** — there is no default system allocator
to fall back on, since there's no OS underneath providing one. The crate
must supply a static implementing `GlobalAlloc` itself: a hand-written
bump allocator over a fixed memory region, a wrapper around a hardware
memory-management unit, or a published embedded allocator crate
(`embedded-alloc`, `linked_list_allocator`, and similar). Exactly one
`#[global_allocator]` may exist in a binary's final dependency graph, the
same one-per-binary restriction `#[panic_handler]` has, for the same
reason: the linker needs exactly one answer for where allocation calls
go.

A crate that avoids the heap entirely — using only stack-allocated data
and fixed-capacity, non-allocating collections (`heapless::Vec` and
similar) — needs neither `extern crate alloc;` nor
`#[global_allocator]` at all. The attribute only becomes relevant the
moment a `#![no_std]` crate wants `Vec`, `Box`, `String`, or anything else
from `alloc`.

## Basic usage example

```
#![no_std]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut() // a real allocator would return a genuine pointer
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator] // <- every alloc::vec::Vec/Box/String call now goes through DummyAllocator
static ALLOCATOR: DummyAllocator = DummyAllocator;
```

## Best practices & deeper information

### Scenario: Designing a public API

A `#![no_std]` firmware crate wants to use `alloc::vec::Vec` for a
sensor-reading buffer, which requires wiring up a heap allocator first —
here, a simple bump allocator over a fixed static region, since there's
no OS-provided heap to fall back on.

```
#![no_std]
extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

const HEAP_SIZE: usize = 1024;

struct BumpAllocator {
    heap: UnsafeCell<[u8; HEAP_SIZE]>,
    next: AtomicUsize,
}

unsafe impl Sync for BumpAllocator {} // safe: access is serialized via the atomic offset

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let offset = self.next.fetch_add(layout.size(), Ordering::SeqCst);
        if offset + layout.size() > HEAP_SIZE {
            return core::ptr::null_mut(); // out of heap
        }
        unsafe { (self.heap.get() as *mut u8).add(offset) }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocators never reclaim individual allocations
    }
}

#[global_allocator] // <- required: no_std + alloc has no default heap without this
static HEAP: BumpAllocator = BumpAllocator {
    heap: UnsafeCell::new([0; HEAP_SIZE]),
    next: AtomicUsize::new(0),
};
```

**Why this way:** `alloc`'s collection types have no allocator to call
into until one is designated with `#[global_allocator]`; a bump allocator
is the simplest correct implementation for firmware with a small, known
memory budget and no need to free individual allocations — the
[embedded Rust book](https://doc.rust-lang.org/stable/embedded-book/collections/index.html)
covers wiring up `alloc` in `#![no_std]` exactly this way, and recommends
a published allocator crate (`embedded-alloc` or similar) over a
hand-written one for anything beyond a toy example.

## Embedded Rust Notes

**Full support**, and this attribute exists largely for `#![no_std]`
contexts — it's the mechanism that makes `extern crate alloc;` usable at
all on a target with no OS-provided heap. In hosted `std` code it's an
optional performance/instrumentation swap (`jemalloc`, `mimalloc`)
rather than a requirement. See [#![no_std]](no-std-attribute.md) for the
distinction between `core`-only `#![no_std]` code (no heap at all,
`#[global_allocator]` irrelevant) and `#![no_std]` + `alloc` code, where
this attribute becomes mandatory.

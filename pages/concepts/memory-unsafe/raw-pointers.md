---
title: "Raw pointers (*const T / *mut T)"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "Boxing", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, "&", "*"]
see_also: ["Unsafe Rust", "FFI (foreign function interface)", "Smart pointers (Box<T>)", "Borrowing (shared references)", "Memory layout & repr"]
---

## Explanation

A raw pointer — `*const T` for a read-only pointer or `*mut T` for a
mutable one — is an address in memory with no attached guarantees. Unlike
a reference (`&T`/`&mut T`), a raw pointer is allowed to be null, dangle
past the lifetime of what it pointed to, be unaligned, or alias another
mutable pointer to the same memory. The borrow checker does not track raw
pointers at all: creating one is always safe (it's just copying an
address), but dereferencing one requires [`unsafe`](unsafe-rust.md)
precisely because none of the compiler's usual safety guarantees apply.

Raw pointers exist because a handful of real tasks fall outside what
references can express: talking to C, which has no concept of Rust's
borrow rules and represents every pointer this way; building a custom
data structure (an intrusive linked list, a graph with cycles) where the
aliasing or lifetime rules a safe reference would demand are too strict
for the shape being modeled; and squeezing out control over exactly when
a value is read, written, or freed — which is how `Box`, `Vec`, and `Rc`
are implemented internally, underneath the safe API they expose.

The mental model is "an address, plus a promise." The type system tracks
almost nothing about a raw pointer beyond what it points to — it is the
programmer's job to know, at every dereference, that the pointer is
non-null, points to a live, correctly-typed, properly aligned value, and
isn't being aliased in a way that violates Rust's aliasing rules. This is
a strictly weaker set of guarantees than a reference gives, which is why
idiomatic Rust reaches for raw pointers only at the boundary — inside an
unsafe module's implementation, or at an [FFI](ffi.md) call site — and
converts back to a safe reference or owned value as soon as possible.

Raw pointers are also the conversion target for `Box::into_raw` and the
conversion source for `Box::from_raw` — the standard way to hand a heap
allocation to code (often C code) that doesn't know about Rust ownership,
and to reclaim it later. See [Smart pointers (Box<T>)](../ownership-borrowing/smart-pointers-box.md)
for the owning side of that relationship.

## Basic usage example

```
let x = 10;
let ptr: *const i32 = &x; // <- creating a raw pointer is always safe

unsafe {
    // SAFETY: `ptr` was derived from the live reference `&x` on the line
    // above, so it points to a valid, initialized i32.
    println!("{}", *ptr); // <- dereferencing requires unsafe
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

C APIs express "pointer to a mutable buffer" and "pointer to read-only
data" as `*mut T`/`*const T`, so any Rust function calling into one has
to convert its safe references at the boundary.

```
unsafe extern "C" {
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
}

fn copy_sensor_frame(src: &[u8], dest: &mut [u8]) {
    assert!(dest.len() >= src.len());
    unsafe {
        // SAFETY: both slices are valid Rust references for their full
        // length, `dest` is at least as long as `src`, and neither
        // aliases the other since they come from two distinct bindings.
        memcpy(dest.as_mut_ptr(), src.as_ptr(), src.len()); // <- *mut u8 / *const u8 cross the FFI boundary
    }
}
```

**Why this way:** C has no concept of a Rust reference, so `as_ptr()`/
`as_mut_ptr()` are the standard conversion at the FFI edge — the
[Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html) documents raw
pointers as the lingua franca for exchanging buffers with foreign code,
with the caller responsible for upholding the length and aliasing
invariants the C function assumes.

### Scenario: Boxing and heap allocation

Handing a heap allocation to code that will hold onto it outside of
Rust's ownership tracking — a C callback registry, a manually managed
cache slot — means converting the `Box` into a raw pointer and back.

```
struct SensorConfig {
    sample_rate_hz: u32,
}

let boxed = Box::new(SensorConfig { sample_rate_hz: 200 });
let raw: *mut SensorConfig = Box::into_raw(boxed); // <- ownership becomes a raw pointer; no Drop runs yet

unsafe {
    // SAFETY: `raw` came from `Box::into_raw` above, was never freed in
    // between, and is reclaimed here exactly once.
    let reclaimed = Box::from_raw(raw); // <- raw pointer converted back into an owning Box
    println!("{} Hz", reclaimed.sample_rate_hz);
} // reclaimed drops here, freeing the allocation
```

**Why this way:** `Box::into_raw` suspends automatic cleanup so the
pointer can be stored somewhere Rust's ownership rules don't reach (a C
struct field, a `static`), and `Box::from_raw` is the only way to give
that memory back to Rust's allocator — the
[`Box` documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw)
requires calling `from_raw` exactly once per `into_raw`'d pointer, since
calling it twice double-frees and never calling it leaks.

### Scenario: Designing a public API

A safe wrapper around a raw-pointer-based structure should never let a
caller construct or dereference the pointer directly — the invariant
lives entirely inside the module.

```
pub struct RingBuffer {
    data: *mut u8, // <- raw pointer stays private; never exposed to callers
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut buf = vec![0u8; capacity].into_boxed_slice();
        let data = buf.as_mut_ptr();
        std::mem::forget(buf); // ownership transferred to this struct's Drop impl
        Self { data, capacity }
    }

    pub fn write_byte(&mut self, offset: usize, value: u8) {
        assert!(offset < self.capacity);
        unsafe {
            // SAFETY: `offset < self.capacity` was just checked, and
            // `data` was allocated with exactly `capacity` bytes in `new`.
            *self.data.add(offset) = value; // <- pointer arithmetic + write, both require unsafe
        }
    }
}

impl Drop for RingBuffer {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: `data`/`capacity` describe exactly the allocation
            // `new` created via `into_boxed_slice`, and this reconstructs
            // it exactly once, on the only path that frees it.
            drop(Box::from_raw(std::slice::from_raw_parts_mut(self.data, self.capacity)));
        }
    }
}
```

**Why this way:** keeping the pointer field private means the only code
that ever dereferences it is the handful of methods on `RingBuffer`
itself, which is the "contain unsafety in small modules" idiom from the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
— every external caller only ever sees the safe `write_byte` API.

## Embedded Rust Notes

**Full support.** Raw pointers are core-language and are, if anything,
*more* central in embedded code than hosted code: a memory-mapped
peripheral register is accessed as a raw pointer to a fixed address
(`0x4000_0000 as *mut u32`), and volatile reads/writes to hardware go
through `core::ptr::read_volatile`/`write_volatile` rather than an
ordinary dereference, since the compiler must not reorder or elide
accesses to hardware state the way it safely could for plain memory.

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

## Explanation (Embedded)

Raw pointers are, if anything, more central in embedded code than
anywhere else in the language: a memory-mapped peripheral register has
no safe Rust type of its own — from the compiler's point of view it's
just an address the vendor's datasheet assigns meaning to, and a raw
pointer (`*const u32`/`*mut u32` at, say, `0x4001_0800`) is the only
vocabulary for talking about it directly. Dereferencing that pointer
still needs `unsafe`, exactly as on the classic page, but a *plain*
dereference is the wrong tool even inside the `unsafe` block: `*ptr` and
`*ptr = value` are ordinary memory operations as far as the optimizer is
concerned, and the optimizer is free to reorder them, merge repeated
reads into one, or delete a write it decides is never observed — all
transformations that are perfectly sound for RAM but silently wrong for
a hardware register, where a write can trigger a physical side effect
(an enable bit, a DMA kickoff) and a read can consume a value that will
never be seen again (a UART data register, an event flag that
self-clears on read). `core::ptr::read_volatile`/`write_volatile` (or
the equivalent methods on the raw pointer itself) tell the compiler that
this specific access must happen, exactly once, at exactly this point in
program order — never elided, never reordered past another volatile
access, never coalesced with a neighbor. See [The undefined-behavior
boundary](the-undefined-behavior-boundary.md) for what goes wrong when
that guarantee is skipped.

The other embedded-specific wrinkle is how the pointer itself gets
built. A single hard-coded address cast straight to `*mut u32` is the
simplest form and common for a one-off register; a whole peripheral is
more often modeled as a `#[repr(C)]` struct overlaying its register
block (see [Memory layout & repr](memory-layout-and-repr.md)), with a
raw pointer to the struct's base address and field access reaching each
register at its correct offset. Either way,
[`unsafe`](../../syntax/keywords/unsafe.md) is what a raw peripheral
pointer forces at the point of use, and [`&raw const`/`&raw
mut`](../../syntax/operators/raw-borrow.md) is the tool for taking a
pointer to one field of that struct (or to a `static mut` shared with an
interrupt handler) without asserting a reference to the whole thing is
momentarily exclusive.

## Basic usage example (Embedded)

```
const GPIOA_IDR: *const u32 = 0x4001_0800 as *const u32; // <- fixed peripheral register address, from the datasheet

fn read_port_a() -> u32 {
    unsafe {
        // SAFETY: GPIOA_IDR is a valid, always-mapped peripheral register
        // on this chip; a plain volatile read has no aliasing concerns.
        core::ptr::read_volatile(GPIOA_IDR) // <- volatile: the optimizer must not cache or elide this read
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

Setting one bit of a GPIO output register without disturbing its
neighbors is a read-modify-write through a raw pointer — every step
needs `unsafe`, and every access needs to be volatile or the compiler is
free to fold the read and write together.

```
const GPIOA_ODR: *mut u32 = 0x4001_0814 as *mut u32; // <- output data register

fn set_pin5_high() {
    unsafe {
        // SAFETY: GPIOA_ODR is a valid, word-aligned peripheral register;
        // read-modify-write is the documented way to flip one bit.
        let current = core::ptr::read_volatile(GPIOA_ODR); // <- must observe the register's real current state
        core::ptr::write_volatile(GPIOA_ODR, current | (1 << 5)); // <- must actually reach hardware, not get optimized away
    }
}
```

**Why this way:** a non-volatile `*GPIOA_ODR` read/write pair is legal
Rust but unsound firmware — nothing stops the optimizer from proving the
read is "redundant" with a previous one and reusing a stale value, which
silently drops bits other code set on the same register; the [Rust
embedded book](https://docs.rust-embedded.org/book/start/registers.html)
documents `read_volatile`/`write_volatile` as mandatory for exactly this
reason.

### Scenario: Designing a public API

A GPIO peripheral's raw base address should never leak past the module
that owns it — a safe wrapper takes the address once, and every caller
afterward only sees typed, safe methods.

```
pub struct GpioA {
    base: *mut u32,
}

impl GpioA {
    /// # Safety
    /// Caller must guarantee no other `GpioA` exists for this peripheral.
    pub unsafe fn new(base_address: usize) -> Self {
        Self { base: base_address as *mut u32 }
    }

    pub fn set_pin_high(&self, pin: u8) {
        unsafe {
            // SAFETY: `self.base` was validated as the peripheral's real
            // address when this GpioA was constructed.
            let odr = self.base.add(0x14 / 4); // <- pointer arithmetic to the ODR register's offset
            let current = core::ptr::read_volatile(odr);
            core::ptr::write_volatile(odr, current | (1 << pin));
        }
    }
}
```

**Why this way:** keeping `base` private and the constructor `unsafe`
while every method afterward is safe is the "thin unsafe core, safe
API" idiom applied to hardware — application code calls
`gpio.set_pin_high(5)` and never touches a raw pointer or an address
literal itself.

### Scenario: Crossing an FFI boundary

A DMA controller is programmed by writing a raw source address into one
of its registers — the address itself is a `*const u8` converted to the
integer the peripheral's register expects, not a value the DMA hardware
understands as a Rust type.

```
const DMA_SRC_ADDR_REG: *mut u32 = 0x4002_6000 as *mut u32;

fn start_transfer(buffer: &[u8]) {
    let src_addr = buffer.as_ptr() as u32; // <- raw pointer collapsed to the integer the DMA register stores
    unsafe {
        // SAFETY: `buffer` outlives the DMA transfer this starts, and the
        // DMA controller is idle before this write per the caller's contract.
        core::ptr::write_volatile(DMA_SRC_ADDR_REG, src_addr);
    }
}
```

**Why this way:** the DMA peripheral has no concept of a Rust slice or
reference — only a raw address — so `as_ptr()` followed by an integer
cast is the same "give the foreign side a plain address" idiom as any
FFI boundary, just with a hardware block instead of a C function on the
other end.

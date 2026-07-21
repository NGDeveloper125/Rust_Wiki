---
title: "Const generics"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Declarative / Metaprogramming", "Writing Generic & Reusable Code", "Unique to Rust", "Generic Programming", "Coming from C / C++"]
related_syntax: [const]
see_also: ["Generics", "Associated types"]
---

## Explanation

Const generics parameterize a type by a *value*, not just another type —
most commonly an array length. For example, a
`Buffer<const N: usize> { data: [u8; N] }` struct can be instantiated as
`Buffer<64>`, with `N` becoming part of the type itself.

Before const generics existed, array length wasn't something generic code
could abstract over at all — `[T; N]` for different `N` were unrelated
types with no shared generic interface, forcing either code duplication
per size or falling back to a heap-allocated `Vec` even when a fixed size
was known and stack allocation would have been possible and faster. Const
generics close that gap: `N` is checked and resolved entirely at compile
time, the same way a type parameter is, so `Buffer<64>` and `Buffer<128>`
are monomorphized into separate, specialized code paths with no runtime
cost for the abstraction.

This maps closely to what C++ templates have long allowed with
non-type template parameters, but with Rust's stricter compile-time
checking of what operations on `N` are actually valid.

## Basic usage example

```
fn sum<const N: usize>(arr: [i32; N]) -> i32 { // <- N is a value, known at compile time
    arr.iter().sum()
}

sum([1, 2, 3]);      // N = 3, inferred from the array literal
sum([1, 2, 3, 4]);    // N = 4, a distinct monomorphized instantiation
```

**Restriction:** `N` must be resolvable at compile time — it can be a
literal, a `const`, or inferred from context, but never a value computed
at runtime (like a `Vec`'s length).

## Best practices & deeper information

### Scenario: Writing generic code

A fixed-size buffer type parameterized over its length lets the compiler
catch a capacity mismatch at compile time, and stack-allocate the backing
array, instead of needing a heap-allocated `Vec` just to carry a length
that's actually known up front.

```
struct RingBuffer<const N: usize> { // <- N is part of the type: RingBuffer<8> and RingBuffer<256> differ
    data: [u8; N],
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    fn new() -> Self {
        RingBuffer { data: [0; N], len: 0 }
    }

    fn capacity(&self) -> usize {
        N // <- N is an ordinary compile-time constant inside the impl, not a runtime field
    }
}

let small: RingBuffer<8> = RingBuffer::new();
let large: RingBuffer<256> = RingBuffer::new();
```

**Why this way:** encoding the capacity in the type itself means passing
a `RingBuffer<8>` where a `RingBuffer<256>` is expected is a compile
error rather than a bug discovered at runtime — the same compile-time
guarantee [generics](generics.md) give for types, extended to a value.

## Explanation (Embedded)

Const generics are not just *compatible* with no-heap embedded code —
they are the mechanism that makes no-heap collections work at all.
`heapless::Vec<T, N>`, `heapless::String<N>`, `heapless::spsc::Queue<T, N>`,
and every other fixed-capacity collection in that ecosystem are ordinary
generic structs parameterized by a const `N`, storing their backing data
as `[MaybeUninit<T>; N]` (or equivalent) inline — on the stack or in
`static` memory — plus a small runtime field tracking how much of that
fixed space is currently in use. Without const generics, a crate wanting
a "growable, but bounded, no-heap" type would have no way to express the
bound in the type system at all: either every possible capacity needs its
own hand-written struct, or the capacity becomes a runtime-only
convention nobody's checking, which is exactly the unenforced-assumption
problem const generics close.

Because `N` is part of the type, mismatched capacities are caught at
compile time, at the call site, rather than surfacing as a runtime panic
or a silently truncated buffer on hardware where a debugger may not even
be attached: a function expecting a `heapless::Vec<u8, 64>` genuinely
cannot be called with a `heapless::Vec<u8, 32>`, the same way `Buffer<64>`
and `Buffer<128>` are unrelated types on the classic Explanation. Each
distinct `N` monomorphizes into its own specialized code, so a `[u8; 64]`
DMA buffer and a `[u8; 256]` one cost nothing beyond the storage each
actually occupies — there is no generic runtime indirection paying for
the abstraction.

## Basic usage example (Embedded)

```
struct RxBuffer<const N: usize> { // <- N fixed at compile time: no allocator, no runtime capacity check
    data: [u8; N],
    len: usize,
}

impl<const N: usize> RxBuffer<N> {
    fn new() -> Self {
        RxBuffer { data: [0; N], len: 0 }
    }

    fn push_byte(&mut self, byte: u8) -> Result<(), u8> {
        if self.len == N {
            return Err(byte); // <- buffer full: caught here, not by overrunning a fixed-size array
        }
        self.data[self.len] = byte;
        self.len += 1;
        Ok(())
    }
}

let mut uart_rx: RxBuffer<64> = RxBuffer::new(); // <- 64-byte receive buffer, stack/static storage only
uart_rx.push_byte(0xAA).ok();
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A UART receive buffer's right size depends on the peripheral and use
case — a console UART might need 256 bytes, a low-rate sensor link only
16 — so writing the buffer type once, generic over its capacity, avoids
either hand-duplicating a struct per size or falling back to a heap that
may not exist on the target at all.

```
struct RingBuffer<const N: usize> { // <- same struct serves every peripheral instance, at whatever size it needs
    data: [u8; N],
    head: usize,
    tail: usize,
}

impl<const N: usize> RingBuffer<N> {
    fn new() -> Self {
        RingBuffer { data: [0; N], head: 0, tail: 0 }
    }
}

let console_rx: RingBuffer<256> = RingBuffer::new(); // <- sized for a chatty console UART
let sensor_rx: RingBuffer<16> = RingBuffer::new();   // <- sized for a low-rate sensor link, same code
```

**Why this way:** each instantiation is monomorphized into its own
specialized, stack/`static`-allocated code path, so picking a larger `N`
for the console UART costs exactly the extra bytes of storage it uses —
nothing is paid in indirection or runtime capacity tracking that a
hand-written per-size struct wouldn't already need.

### Scenario: Designing a public API

A driver crate that hands back a fixed-capacity buffer type should encode
the capacity in the return type itself, so a caller wiring the wrong-size
buffer into a peripheral expecting a specific length fails to compile
instead of corrupting a transfer at runtime.

```
fn make_spi_tx_buffer<const N: usize>() -> heapless::Vec<u8, N> {
    heapless::Vec::new() // <- capacity N is part of the returned type, decided by the caller's type annotation
}

fn send_frame(bus: &mut impl embedded_hal::spi::SpiBus, frame: heapless::Vec<u8, 32>) {
    let _ = (bus, frame); // a 32-byte-capacity frame, enforced by the parameter's own type
}

let frame: heapless::Vec<u8, 32> = make_spi_tx_buffer(); // <- N inferred as 32 from send_frame's parameter type
```

**Why this way:** encoding the capacity in the type means a
`heapless::Vec<u8, 16>` accidentally passed where a `heapless::Vec<u8, 32>`
is required is rejected by the compiler, the same guarantee the classic
Explanation describes for `RingBuffer<8>` vs `RingBuffer<256>` — on
hardware where a buffer-size mismatch is a silent memory-safety bug
rather than a caught exception, that compile-time check is doing real
work.

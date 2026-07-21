---
title: "#[repr(...)]"
kind: attribute
embedded_support: full
groups: ["Types & Layout", "Types & Data Structures", "Memory & Unsafe / FFI"]
related_concepts: ["Memory layout & repr", "FFI (foreign function interface)", Structs, "Enums (algebraic data types)"]
related_syntax: [struct, enum, union]
see_also: ["Memory layout & repr"]
---

## Explanation

`#[repr(...)]` is placed directly above a `struct`, `enum`, or `union`
item and takes one or more comma-separated arguments inside its
parentheses, each requesting a specific layout guarantee. Its position
relative to `#[derive(...)]` or doc comments on the same item doesn't
affect its meaning — only that it sits directly above the item it
applies to, with no blank statement between.

`#[repr(Rust)]` is the implicit default and is rarely written out
explicitly — it leaves field order, padding, and enum discriminant width
entirely up to the compiler. `#[repr(C)]` requests the layout a C
compiler would produce for an equivalent struct/union/enum: fields in
declaration order, padded only as C's alignment rules require. `#[repr(transparent)]`
is legal only on a struct or single-variant enum with **exactly one**
field of non-zero size (any other fields must be zero-sized, such as
`PhantomData`); it guarantees the type has identical layout, size, and
ABI to that one field. `#[repr(packed)]` (or `#[repr(packed(N))]` to name
an explicit maximum alignment) removes inter-field padding entirely,
setting alignment to `1` (or `N`) — legal only on `struct`.

On a fieldless (C-like) `enum`, an integer repr — `#[repr(u8)]`,
`#[repr(i32)]`, and so on — fixes the discriminant to that exact integer
type instead of leaving its width to the compiler. An enum whose variants
carry data can combine an integer repr with `C`, written as
`#[repr(C, u8)]`, to get both a C-compatible tagged-union layout and a
specific tag width. Multiple modifiers are always written inside one
attribute, comma-separated — `#[repr(C, packed)]` — never as two separate
`#[repr(...)]` attributes stacked on the same item. `#[repr(align(N))]`
(for a power-of-two `N`) raises a type's minimum alignment and can
likewise be combined with `C`.

**Restriction:** dereferencing a reference to a field inside a
`#[repr(packed)]` type is undefined behavior whenever that field's
natural alignment no longer holds — the packed layout can leave a field
at an address its type wouldn't normally permit. Read such fields by
value (`let x = my_packed.field;`, which copies out first) or through
`std::ptr::addr_of!`, never by taking `&my_packed.field` directly; the
[Rustonomicon](https://doc.rust-lang.org/nomicon/other-reprs.html) covers
this hazard as the main reason `packed` is reached for only when a wire
format genuinely requires zero padding.

The full case for *why* to choose one `repr` over another — and the
layout guarantees each one makes — is covered on the
[Memory layout & repr](../../concepts/memory-unsafe/memory-layout-and-repr.md)
concept page; this page covers the attribute's grammar and legal
combinations.

## Usage examples

### Requesting a C-compatible layout

```
#[repr(C)] // <- requests a stable, C-compatible layout instead of the default repr(Rust)
struct Point {
    x: f32,
    y: f32,
}
```

### Crossing an FFI boundary

A struct shared with a C library often needs more than one layout
guarantee at once — here, a C-compatible field order plus zero padding,
because the C header packs its struct tightly.

```
#[repr(C, packed)] // <- two modifiers in one attribute: C field order AND no padding
#[derive(Clone, Copy)]
struct FrameHeader {
    version: u8,
    flags: u8,
    payload_len: u16,
}

fn payload_len(header: &FrameHeader) -> u16 {
    let header_copy = *header; // read the whole packed value by copy first...
    header_copy.payload_len // <- ...then access: avoids ever taking a reference into packed, misaligned data
}
```

`#[repr(C, packed)]` combines both modifiers in a
single attribute — writing `#[repr(C)] #[repr(packed)]` as two separate
attributes on the same item is legal but conflates two independent
requests where one attribute reads as a single, deliberate layout
decision; the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)
documents `repr` modifiers as combinable within one attribute for exactly
this reason.

### Bit manipulation and flags

A protocol's status byte maps directly onto a fieldless enum; pinning the
discriminant width with an integer repr, alongside explicit discriminant
values, keeps the enum's on-the-wire representation an exact match for
the byte the protocol defines.

```
#[repr(u8)] // <- fixes the discriminant to exactly one byte
enum LinkStatus {
    Down = 0,        // <- explicit discriminants alongside the repr, not required but common together
    Negotiating = 1,
    Up = 2,
}

let byte: u8 = LinkStatus::Up as u8; // <- `as` reads the u8 discriminant `#[repr(u8)]` guarantees
```

Without an integer repr, the compiler is free to pick
whatever discriminant width it wants for a field-less enum — the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#reprc-enums)
specifies `#[repr(u8)]` and its siblings as the way to pin that width
down so a cast to the integer type is guaranteed stable across compiler
versions.

### Designing a public API

A newtype wrapper meant to cross an ABI boundary as if it were its inner
type needs `#[repr(transparent)]` specifically — not `#[repr(C)]`, which
doesn't make the same single-field guarantee.

```
#[repr(transparent)] // <- guarantees identical layout to the single u32 field; repr(C) alone would not
pub struct SensorId(u32);
```

`#[repr(C)]` on a single-field tuple struct happens to
produce the same layout in practice, but only `#[repr(transparent)]`
makes it a guarantee the compiler enforces — attempting it on a type with
more than one non-zero-sized field is a compile error, which is the
signal that the type no longer qualifies, per the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation).

## Explanation (Embedded)

`#[repr(...)]` is arguably as central to embedded Rust as
[`#[cfg(...)]`](cfg-attribute.md), because it's the mechanism that pins a
Rust type's memory layout to a hardware memory map that Rust itself has
no say over. A microcontroller's peripherals expose their registers at
fixed byte offsets from a base address, documented in the vendor's
datasheet/SVD file — a `GPIOA` register block might place `MODER` at
offset `0x00`, `OTYPER` at `0x04`, `IDR` at `0x10`, and so on. A Rust
struct meant to overlay that block field-for-field only overlays it
correctly if the compiler is told not to reorder or pad those fields the
way `#[repr(Rust)]` (the implicit default) is explicitly free to do —
`#[repr(C)]` is what makes the struct's actual layout something the code
can rely on: fields in declaration order, at the memory-mapped offsets
the register block was already documented as having. This is exactly
what `svd2rust` — the tool that generates peripheral-access crates (PACs)
from a chip's SVD file — emits: nearly every register-block struct in a
generated PAC crate carries `#[repr(C)]` for this reason, and
reading/writing a peripheral through the generated struct only works
because that layout guarantee holds.

`#[repr(transparent)]` shows up constantly in the same neighborhood for a
different reason: wrapping a single raw register value (a `u32`, say) in
a newtype for type safety — a `Millivolts(u32)` or `GpioPin(u8)` —
without paying any cost for the wrapper. Because `#[repr(transparent)]`
guarantees identical layout, size, and ABI to the single inner field, the
newtype can be passed to/from an `extern "C"` HAL function, transmuted
from a raw memory-mapped read, or stored directly in a register-block
field with zero runtime overhead — it's a genuinely zero-cost abstraction
specifically because the repr makes that guarantee load-bearing rather
than incidental.

`#[repr(packed)]` also appears more often in embedded code than in most
hosted code, for the same reason it does in FFI: a radio or bus protocol
frame that must match an exact byte layout with no compiler-inserted
padding.

## Usage examples (Embedded)

### A peripheral register block laid out to match a datasheet's memory map

```
#[repr(C)] // <- pins field order/offsets to match the GPIOA register block's documented layout
pub struct GpioARegisterBlock {
    pub moder: u32,   // offset 0x00
    pub otyper: u32,  // offset 0x04
    pub ospeedr: u32, // offset 0x08
    pub idr: u32,     // offset 0x10
}

const GPIOA_BASE: usize = 0x4002_0000;

fn gpioa() -> &'static mut GpioARegisterBlock {
    unsafe { &mut *(GPIOA_BASE as *mut GpioARegisterBlock) }
}
```

### A zero-cost newtype around a raw register value

```
#[repr(transparent)] // <- guarantees identical layout/ABI to the inner u16, zero cost over the raw value
pub struct AdcReading(u16);

impl AdcReading {
    pub fn millivolts(self, vref_mv: u32) -> u32 {
        (self.0 as u32 * vref_mv) / 4095
    }
}
```

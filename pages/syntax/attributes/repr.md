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

## Basic usage example

```
#[repr(C)] // <- requests a stable, C-compatible layout instead of the default repr(Rust)
struct Point {
    x: f32,
    y: f32,
}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

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

**Why this way:** `#[repr(C, packed)]` combines both modifiers in a
single attribute — writing `#[repr(C)] #[repr(packed)]` as two separate
attributes on the same item is legal but conflates two independent
requests where one attribute reads as a single, deliberate layout
decision; the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)
documents `repr` modifiers as combinable within one attribute for exactly
this reason.

### Scenario: Bit manipulation and flags

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

**Why this way:** without an integer repr, the compiler is free to pick
whatever discriminant width it wants for a field-less enum — the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#reprc-enums)
specifies `#[repr(u8)]` and its siblings as the way to pin that width
down so a cast to the integer type is guaranteed stable across compiler
versions.

### Scenario: Designing a public API

A newtype wrapper meant to cross an ABI boundary as if it were its inner
type needs `#[repr(transparent)]` specifically — not `#[repr(C)]`, which
doesn't make the same single-field guarantee.

```
#[repr(transparent)] // <- guarantees identical layout to the single u32 field; repr(C) alone would not
pub struct SensorId(u32);
```

**Why this way:** `#[repr(C)]` on a single-field tuple struct happens to
produce the same layout in practice, but only `#[repr(transparent)]`
makes it a guarantee the compiler enforces — attempting it on a type with
more than one non-zero-sized field is a compile error, which is the
signal that the type no longer qualifies, per the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation).

## Embedded Rust Notes

**Full support.** `#[repr(...)]` is core-language and works identically
without `std` — embedded code depends on it constantly for memory-mapped
peripheral structs (`#[repr(C)]` register blocks matching a
manufacturer's datasheet byte-for-byte) and packed protocol frames.
`#[repr(packed)]` in particular shows up far more in embedded contexts
than hosted ones, since removing all padding to match a wire format is
more often worth the unaligned-access cost on a microcontroller than on a
desktop CPU.

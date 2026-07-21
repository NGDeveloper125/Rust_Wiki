---
title: "Memory layout & repr"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "FFI / Interop", "Unique to Rust", "Coming from C / C++"]
related_syntax: [struct, enum, "#[repr(...)]"]
see_also: ["Unsafe Rust", "Raw pointers (*const T / *mut T)", "FFI (foreign function interface)", "Structs", "Enums (algebraic data types)"]
---

## Explanation

Memory layout is the answer to "given a value of this type, exactly which
bytes represent it, in what order, with what padding between fields, and
how big is the whole thing?" By default, Rust deliberately leaves almost
all of this unspecified: the compiler is free to reorder a struct's
fields, insert padding for alignment, or shrink an enum's tag however it
likes, because doing so often produces a smaller or faster layout than
declaration order would. This default is called `repr(Rust)`, and it is
not a stable, documented format — two different compiler versions are
allowed to lay the same struct out differently.

The `#[repr(...)]` attribute is how a type opts out of that freedom and
requests a specific, guaranteed layout instead. `#[repr(C)]` is the most
common case: it lays a struct out exactly the way a C compiler would —
fields in declaration order, with only the padding C's alignment rules
require — which is what makes the type safe to hand across an [FFI](ffi.md)
boundary, where the other side has no idea what `repr(Rust)` even is.
Other forms exist for narrower needs: `#[repr(transparent)]` guarantees a
single-field wrapper has exactly the same layout as its one field (the
mechanism behind the [newtype pattern](../types-data-modeling/the-newtype-pattern.md)
being FFI-safe), and `#[repr(u8)]`/`#[repr(u32)]`/etc. fix the integer
type backing a field-less enum's discriminant, which matters for
hardware register values and wire protocols as much as for C interop.

The mental model is: layout is an implementation detail *unless* you say
otherwise, and saying otherwise is a promise the compiler will now keep
forever for that type. Choosing `repr(C)` over `repr(Rust)` is not a
performance decision — `repr(Rust)` is often smaller, thanks to field
reordering and niche optimizations the compiler performs automatically —
it's a compatibility decision, made only when something outside the Rust
compiler (a C library, a hardware register, a serialized wire format)
needs to agree on the byte-for-byte shape of the type.

`repr` is closely tied to [unsafe Rust](unsafe-rust.md): most code that
cares about layout is also the code dereferencing [raw
pointers](raw-pointers.md) into that memory, transmuting between types,
or reading hardware registers, so getting the `repr` wrong is one of the
more common ways to slip past [the undefined-behavior
boundary](the-undefined-behavior-boundary.md) — a struct assumed to be
`repr(C)` but left as the default `repr(Rust)` can have its fields in a
completely different order than the reader expects.

## Basic usage example

```
#[repr(C)] // <- requests a stable, C-compatible layout instead of the default repr(Rust)
struct Point {
    x: f32,
    y: f32,
}

let p = Point { x: 1.0, y: 2.0 };
println!("{} {}", p.x, p.y);
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A struct shared between Rust and a C sensor-calibration library must have
a layout both languages agree on — the default `repr(Rust)` layout gives
no such guarantee, even if the fields look identical to the C struct.

```
#[repr(C)] // <- required: matches the C struct's field order and padding exactly
pub struct CalibrationData {
    offset: f32,
    scale: f32,
    channel: u8,
}

unsafe extern "C" {
    fn apply_calibration(data: *const CalibrationData); // <- the C side expects repr(C) layout
}

pub fn apply(data: &CalibrationData) {
    unsafe {
        // SAFETY: `data` is a valid, live reference for the duration of
        // this call, and `CalibrationData` is `repr(C)`, matching the
        // layout the C function's header declares.
        apply_calibration(data); // <- FFI call relies on CalibrationData's layout matching C's
    }
}
```

**Why this way:** without `#[repr(C)]`, the compiler is free to reorder
`offset`, `scale`, and `channel` or change their padding, silently
corrupting every field the C side reads — the
[Rustonomicon](https://doc.rust-lang.org/nomicon/other-reprs.html)
documents `repr(C)` as the layout guarantee FFI structs require, precisely
because `repr(Rust)` makes no such promise.

### Scenario: Bit manipulation and flags

A hardware status register read from a device driver is a fixed-width
integer where individual bits carry meaning — `#[repr(u8)]` on a
field-less enum pins the discriminant to the exact width the register
uses, instead of leaving it to the compiler.

```
#[repr(u8)] // <- fixes the discriminant to a single byte, matching the register's width
enum LinkStatus {
    Down = 0,
    Negotiating = 1,
    Up = 2,
}

fn status_from_register(byte: u8) -> Option<LinkStatus> {
    match byte {
        0 => Some(LinkStatus::Down),
        1 => Some(LinkStatus::Negotiating),
        2 => Some(LinkStatus::Up),
        _ => None,
    }
}
```

**Why this way:** without an explicit `repr`, the compiler chooses
whatever discriminant size it wants for a field-less enum, which is fine
until the value must line up with a specific register width or wire
format — the [Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#reprc-enums)
specifies `repr(u8)` and friends exactly for pinning an enum's
discriminant to a known integer type.

### Scenario: Designing a public API

Wrapping a single value in a newtype for domain safety should cost
nothing at runtime — `#[repr(transparent)]` guarantees the wrapper has
identical layout to its inner field, so it's free to pass across an ABI
boundary as if the wrapper didn't exist.

```
#[repr(transparent)] // <- guarantees identical layout to the wrapped u32, not just "probably fine"
pub struct SensorId(u32);

unsafe extern "C" {
    fn lookup_sensor(id: SensorId) -> i32; // <- SensorId is layout-compatible with a plain u32 here
}
```

**Why this way:** `#[repr(transparent)]` is the one guarantee that makes a
newtype ABI-compatible with its single field, which the
[Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation)
specifies precisely so wrapper types keep their compile-time safety
benefits without adding an FFI-visible layout difference.

## Embedded Rust Notes

**Full support.** `repr` is core-language and works identically without
`std` — embedded code depends on it constantly for memory-mapped
peripheral structs (`#[repr(C)]` register blocks matching a
manufacturer's datasheet layout byte-for-byte) and for packed bitfield
representations of protocol frames. `#[repr(packed)]` in particular shows
up more in embedded contexts than hosted ones, since removing all padding
to match a wire format is far more often worth the unaligned-access cost
on a microcontroller than on a desktop CPU.

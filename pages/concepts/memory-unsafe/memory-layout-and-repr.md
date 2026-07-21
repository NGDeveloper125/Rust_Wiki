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

## Explanation (Embedded)

The mechanics of *how* `#[repr(C)]`, `#[repr(transparent)]`, and
`#[repr(packed)]` change a type's layout are covered in full on the
[`#[repr(...)]` syntax page](../../syntax/attributes/repr.md)'s embedded
section, including the `svd2rust`-generated register-block pattern; this
page's job is the broader design point those mechanics serve. A
microcontroller's peripherals are a fixed memory map handed down by the
manufacturer's datasheet (and often an SVD file): `MODER` at offset
`0x00` from the GPIO block's base address, `OTYPER` at `0x04`, `IDR` at
`0x10`, and so on, with no say in the matter for the software reading
them. A Rust struct meant to overlay that block is a claim about
physical reality — "byte N of this struct is register X" — and that
claim is only true if the struct's actual field order, size, and padding
line up with the hardware's offsets exactly.

This is precisely what the default `repr(Rust)` cannot promise: the
compiler is explicitly free to reorder a struct's fields or change
padding between compiler versions, because for an ordinary in-memory
struct that freedom is a pure win (smaller size, better cache behavior)
with nothing external depending on the specific arrangement. A
register-block struct is the opposite case — something external (the
silicon) already fixed the arrangement, so the struct has to match it
instead of the other way around, and `#[repr(C)]`'s promise (declaration
order, only C-style alignment padding) is what makes "field order in the
source" and "offset in the datasheet" the same fact instead of two facts
that happen to agree by luck. Getting this wrong doesn't produce a type
error: a register block with one field reordered still compiles and
still runs, it just silently reads and writes the *wrong register* at
every affected offset, which is exactly the kind of hardware-adjacent
[undefined-behavior boundary](the-undefined-behavior-boundary.md)
mismatch that's expensive to debug because nothing about the Rust code
looks wrong in isolation.

## Basic usage example (Embedded)

```
#[repr(C)] // <- mandatory: pins field order/offsets to the datasheet's memory map
pub struct GpioBlock {
    moder: u32,  // offset 0x00
    otyper: u32, // offset 0x04
    idr: u32,    // offset 0x08
}

const GPIOA: *mut GpioBlock = 0x4002_0000 as *mut GpioBlock;
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A peripheral driver exposes typed, safe methods over a `#[repr(C)]`
register block; the struct's layout is the load-bearing contract the
whole driver depends on, so it's declared once and never touched by
callers directly.

```
#[repr(C)] // <- field order/offsets must match the UART's datasheet layout exactly
pub struct UartRegisters {
    sr: u32,  // status register, offset 0x00
    dr: u32,  // data register, offset 0x04
    brr: u32, // baud rate register, offset 0x08
}

pub struct Uart {
    regs: *mut UartRegisters,
}

impl Uart {
    pub fn is_data_ready(&self) -> bool {
        unsafe {
            // SAFETY: `regs` points at a real, always-mapped UART block.
            (core::ptr::read_volatile(&raw const (*self.regs).sr) & 0x1) != 0
        }
    }
}
```

**Why this way:** every method on `Uart` trusts that `sr`/`dr`/`brr` sit
at offsets `0x00`/`0x04`/`0x08` because `#[repr(C)]` guarantees it —
dropping the attribute would leave the compiler free to reorder these
three `u32` fields, and `is_data_ready` would silently read whichever
register the compiler happened to place first instead of the status
register.

### Scenario: Bit manipulation and flags

Datasheets frequently leave gaps in a peripheral's memory map — reserved
words between two real registers — and a register-block struct has to
model the gap explicitly or every field after it lands at the wrong
offset.

```
#[repr(C)]
pub struct AdcRegisters {
    cr: u32,          // offset 0x00: control register
    _reserved: u32,   // <- offset 0x04 is reserved on this chip; must still occupy the slot
    dr: u32,          // offset 0x08: data register — wrong offset entirely without the gap above
}
```

**Why this way:** `#[repr(C)]` only guarantees C-style padding for
alignment, not for a vendor-defined "reserved" gap that has nothing to
do with alignment — the [Rust
Reference](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)
documents `repr(C)` as laying fields out in declaration order with no
reordering, so an explicit placeholder field is the correct, portable
way to reserve the space rather than relying on incidental alignment
padding to happen to be the right size.

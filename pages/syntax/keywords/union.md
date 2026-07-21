---
title: "union"
kind: keyword
embedded_support: full
groups: ["Types & Data Structures", "Memory & Unsafe / FFI"]
related_concepts: ["FFI (foreign function interface)", "Memory layout & repr", "Unsafe Rust"]
related_syntax: [struct, "unsafe", "#[repr(...)]"]
see_also: ["#[repr(...)]"]
---

## Explanation

`union` declares a type whose grammar looks just like a named-field
`struct` — a name, generics, and a brace-delimited list of `name: Type`
fields — but whose fields all occupy the **same** region of memory,
starting at the same offset. The type's size is the size of its largest
field (plus any padding for alignment), not the sum of all fields the way
a struct's size would be. Unlike `enum`, a `union` stores no hidden
discriminant recording which field was last written — nothing in the
value itself tracks which interpretation is currently valid; that
bookkeeping is entirely the programmer's responsibility, usually via a
convention borrowed from whatever C header the union mirrors.

Writing to a union field is ordinary, safe syntax: `my_union.field =
value;`. **Reading** a union field, by contrast, requires an `unsafe`
block — `unsafe { my_union.field }` — because the compiler has no way to
verify the bytes currently stored actually represent that field's type;
reinterpreting them incorrectly is undefined behavior. Pattern matching on
a union field similarly requires an `unsafe` block around the match.

A `union` can only implement `Copy` for all its fields, or otherwise must
wrap any field needing a non-trivial destructor in
`std::mem::ManuallyDrop<T>` — the compiler cannot know which field to run
`Drop` for at the end of the union's lifetime, so it refuses to derive
`Drop` at all. In practice, `union` is used almost exclusively at an FFI
boundary to bind a C `union` declared in a header, nearly always paired
with `#[repr(C)]` so its layout matches the C compiler's exactly — see
[`#[repr(...)]`](../attributes/repr.md). Idiomatic Rust code reaches for
`enum` instead whenever a tagged, memory-safe alternative is possible,
since an `enum`'s hidden discriminant gives the same "one of several
shapes" behavior with safe reads.

## Basic usage example

```
#[repr(C)]
union RegisterValue {
    as_u32: u32,
    as_bytes: [u8; 4],
}

let value = RegisterValue { as_u32: 0x1234_5678 }; // <- writing a field is safe
let bytes = unsafe { value.as_bytes }; // <- `union` reads require `unsafe`: the compiler can't verify which field is live
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A C library's header declares a `union` alongside a separate tag field
telling callers which member is active; the Rust binding mirrors both the
union's layout and the manual discipline of only reading the field the
tag says is valid.

```
#[repr(C)]
union SensorPayload { // <- `union`: both fields share the same memory
    analog_mv: u16,
    digital_state: bool,
}

#[repr(C)]
struct SensorReading {
    kind: u8, // 0 = analog, 1 = digital -- the tag the C header defines
    payload: SensorPayload,
}

fn describe(reading: &SensorReading) -> String {
    match reading.kind {
        0 => format!("{} mV", unsafe { reading.payload.analog_mv }), // <- unsafe: trusts `kind` to be accurate
        _ => format!("digital: {}", unsafe { reading.payload.digital_state }),
    }
}
```

**Why this way:** because a `union` carries no discriminant of its own,
every read is only as safe as the external tag it's paired with is
accurate — the
[Rust Reference](https://doc.rust-lang.org/reference/items/unions.html)
specifies `union` field access as `unsafe` precisely because the type
system cannot verify that invariant on its own; an `enum` is preferred
whenever the C side doesn't force this shape.

## Embedded Rust Notes

**Full support.** `union` is core-language and allocator-free. Embedded
code uses it for the same reason hosted FFI code does — binding a
vendor's C header that declares a union for a hardware register or
protocol frame — with `#[repr(C)]` (or `#[repr(packed)]` for a
bit-for-bit register layout) applied identically under `#![no_std]`.

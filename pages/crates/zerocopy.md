---
title: "zerocopy"
version: "0.8.57"
publisher: "Jack Wrenn (jswrenn), Josh Liebow-Feeser (joshlf)"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-12"
summary: "Turns bytes into typed values without copying or `unsafe`. Derives prove at compile time that a type can be reinterpreted from a byte slice, so parsing a packet header is a cast rather than a field-by-field read."
categories: ["parsing", "binary", "no-std"]
repository: "https://github.com/google/zerocopy"
---

## Overview

Reading a binary format usually means copying: take four bytes, build a `u32`,
assign it to a field, repeat. For a packet header on a hot path that copying is
the work, and the obvious alternative — casting the byte slice to a `&Header` —
is `unsafe` and easy to get wrong. Padding bytes, invalid bit patterns and
alignment each have their own way of turning it into undefined behaviour.

`zerocopy` makes the cast safe. You derive traits that state what your type
allows, the compiler checks the claims, and the conversions become ordinary safe
calls:

```
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Header {
    version: u8,
    flags: u8,
    length: [u8; 2],
}

let packet = [1u8, 0b0000_0011, 0x01, 0x00, 0xaa, 0xbb];

// A view into the original bytes: no copy, no unsafe.
let (header, body) = Header::ref_from_prefix(&packet).unwrap();

assert_eq!(header.version, 1);
assert_eq!(u16::from_le_bytes(header.length), 1);
assert_eq!(body, &[0xaa, 0xbb]);
```

**The traits are the design, and each one names a distinct hazard:**

- **`FromBytes`** — every bit pattern of this size is a valid value. True for
  integers and arrays of them; false for `bool`, `char` and most enums, where
  some patterns are invalid.
- **`IntoBytes`** — the type has no padding, so exposing it as bytes cannot leak
  uninitialised memory.
- **`Immutable`** — contains no interior mutability, so a shared reference
  really is read-only.
- **`KnownLayout`** — the layout is knowable, which is what allows slices and
  trailing dynamically-sized fields.
- **`Unaligned`** — alignment is 1, so the cast works at any offset.

A derive that cannot prove its claim is a compile error, which is the point: the
`unsafe` reasoning happens once, in the crate, rather than at every call site.

**Alignment is the thing that trips people.** `ref_from_bytes` borrows the
original bytes, so the type's alignment must be satisfied by the slice's
address — and a buffer read off a socket is aligned to nothing in particular. Two
ways out: make the type `Unaligned` (store multi-byte fields as byte arrays or
as `zerocopy::byteorder` types), or use `read_from_bytes`, which copies and
therefore doesn't care. The first is genuinely zero-copy; the second is still
better than hand-rolled parsing.

`bytemuck` is the neighbour worth knowing: a similar idea with a smaller,
simpler surface aimed at plain-old-data casts, especially for graphics buffers.
`zerocopy` goes further — unaligned access, dynamically-sized types, fallible
`TryFromBytes` for types with invalid patterns — at the cost of more traits to
learn. It is `no_std`, has no required dependencies, requires Rust 1.56, and is
maintained at Google with the `unsafe` core independently reviewed.

## When to use it

### Use case: Parsing a network packet header

The archetypal case. Bytes arrive from a socket; the header is a fixed layout,
and a real parser would copy every field.

```
use zerocopy::byteorder::network_endian::U16;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug)]
#[repr(C)]
struct UdpHeader {
    src_port: U16, // <- big-endian on the wire, whatever this machine is
    dst_port: U16,
    length: U16,
    checksum: U16,
}

let datagram = [0x00, 0x35, 0x04, 0x01, 0x00, 0x0c, 0xab, 0xcd, b'h', b'i'];
let (header, payload) = UdpHeader::ref_from_prefix(&datagram).unwrap();

assert_eq!(header.src_port.get(), 53);
assert_eq!(header.dst_port.get(), 1025);
assert_eq!(payload, b"hi");
```

**Why it fits:** the header is a view into the datagram, so parsing is a bounds
check and a pointer offset. The `U16` type carries its endianness, so `.get()`
does the byte swap and the struct stays `Unaligned` — which is what lets it
apply to a buffer at any address.

### Use case: Writing a struct out as bytes

The same machinery in reverse, for producing a wire format or a file header.

```
use zerocopy::byteorder::little_endian::U32;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct FileHeader {
    magic: [u8; 4],
    version: U32,
    entries: U32,
}

let header = FileHeader {
    magic: *b"WIKI",
    version: U32::new(2),
    entries: U32::new(17),
};

let bytes = header.as_bytes();
assert_eq!(bytes.len(), 12);
assert_eq!(&bytes[..4], b"WIKI");
assert_eq!(&bytes[4..8], &[2, 0, 0, 0]); // <- little-endian

// And straight back again.
let parsed = FileHeader::ref_from_bytes(bytes).unwrap();
assert_eq!(parsed.entries.get(), 17);
```

**Why it fits:** one struct definition serves both directions, so the read and
write paths cannot disagree about the layout. `IntoBytes` will not derive if the
struct has padding, which is the compiler refusing to let you write
uninitialised bytes to a file.

### Use case: Viewing a buffer as a slice of records

A file of fixed-size entries — an index, a sample buffer, a page table — can be
addressed as a slice without building a `Vec`.

```
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned, Debug, PartialEq)]
#[repr(C)]
struct Entry {
    id: [u8; 2],
    kind: u8,
    flags: u8,
}

let buffer = [1u8, 0, 7, 0, 2, 0, 9, 1];

let entries = <[Entry]>::ref_from_bytes(&buffer).unwrap();

assert_eq!(entries.len(), 2);
assert_eq!(entries[0], Entry { id: [1, 0], kind: 7, flags: 0 });
assert_eq!(entries[1].kind, 9);
```

**Why it fits:** the whole file becomes a `&[Entry]` in one bounds-checked step,
and indexing it costs nothing. Collecting into a `Vec<Entry>` would copy every
record, which for a memory-mapped index is the difference between instant and
slow.

## API map

The crate is five derives and the conversion methods they unlock. The derives
are what you write; the methods below are what becomes available once a type has
them.

### The derives

#### `FromBytes`

Asserts every bit pattern is valid, which is what allows bytes to become a
value.

```
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C)]
struct Sample {
    left: i16,
    right: i16,
}

// Integers and arrays of them qualify; bool and char do not.
let bytes = [0x01, 0x00, 0xff, 0xff];
let sample = Sample::read_from_bytes(&bytes[..]).unwrap();

assert_eq!(sample.left, 1);
assert_eq!(sample.right, -1);
```

**When to use it:** on any type you build from bytes. If the derive fails, a
field has invalid bit patterns — a `bool`, a `char`, an enum — and
`TryFromBytes` is the fallible alternative that checks at runtime instead.

#### `IntoBytes`

Asserts the type has no padding, so its bytes are all initialised.

```
use zerocopy::{Immutable, IntoBytes};

#[derive(IntoBytes, Immutable)]
#[repr(C)]
struct Packed {
    a: u16,
    b: u16, // <- same size, so no padding between them
}

let value = Packed { a: 0x0201, b: 0x0403 };
assert_eq!(value.as_bytes(), &[0x01, 0x02, 0x03, 0x04]);
```

**When to use it:** on anything you serialise. A struct with a `u8` followed by
a `u32` has three padding bytes and will not derive — reorder the fields, or add
explicit padding you control, because those bytes would otherwise be whatever
was in that memory before.

#### `KnownLayout` and `Immutable`

The two supporting claims: the layout can be computed, and there is no interior
mutability.

```
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C)]
struct Frame {
    len: u8,
    data: [u8; 3],
}

let bytes = [3u8, b'a', b'b', b'c'];
let frame = Frame::ref_from_bytes(&bytes).unwrap();

assert_eq!(frame.len, 3);
assert_eq!(&frame.data, b"abc");
```

**When to use it:** derive both alongside `FromBytes` as a matter of course.
`KnownLayout` is what makes `ref_from_*` and slice casts possible; `Immutable`
is required by the methods taking `&self`, since a `Cell` inside would let the
bytes change while something holds a reference to them.

#### `Unaligned`

Asserts alignment 1, so a cast works at any address.

```
use zerocopy::{FromBytes, Immutable, KnownLayout, Unaligned};

#[derive(FromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Tag {
    kind: u8,
    value: [u8; 2], // <- bytes, not u16, so alignment stays 1
}

// Works at an odd offset, where an aligned type would fail.
let buffer = [0xff, 7, 1, 2];
let tag = Tag::ref_from_bytes(&buffer[1..]).unwrap();

assert_eq!(tag.kind, 7);
assert_eq!(tag.value, [1, 2]);
```

**When to use it:** any type cast from a buffer you did not allocate — network
data, a memory map, a slice at an arbitrary offset. It is the difference between
`ref_from_bytes` working and failing at runtime on some inputs and not others,
which is a miserable bug to chase.

### Borrowing from bytes

#### `ref_from_bytes`

A reference into the original bytes, requiring an exact size match.

```
use zerocopy::{FromBytes, Immutable, KnownLayout, Unaligned};

#[derive(FromBytes, KnownLayout, Immutable, Unaligned, Debug, PartialEq)]
#[repr(C)]
struct Pair { a: u8, b: u8 }

let bytes = [1u8, 2];
assert_eq!(Pair::ref_from_bytes(&bytes).unwrap(), &Pair { a: 1, b: 2 });

// Too many bytes is an error, not a silent prefix.
assert!(Pair::ref_from_bytes(&[1u8, 2, 3]).is_err());
```

**When to use it:** when the buffer is exactly one value. The strictness is
deliberate — a size mismatch usually means the format is not what you thought,
and silently ignoring the remainder would hide it.

#### `ref_from_prefix`

Takes a value off the front, returning it and the rest.

```
use zerocopy::{FromBytes, Immutable, KnownLayout, Unaligned};

#[derive(FromBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Len { value: [u8; 2] }

let message = [0x03, 0x00, b'a', b'b', b'c'];
let (len, rest) = Len::ref_from_prefix(&message).unwrap();

assert_eq!(u16::from_le_bytes(len.value), 3);
assert_eq!(rest, b"abc");
```

**When to use it:** header-then-body formats, and parsing a stream one record at
a time. Chaining it is how you walk a structured buffer — each call hands you
the remainder to feed the next.

#### `mut_from_bytes`

A mutable view, so writing through it edits the original buffer.

```
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Counter { hits: [u8; 2] }

let mut buffer = [0u8, 0];
let counter = Counter::mut_from_bytes(&mut buffer).unwrap();
counter.hits = 5u16.to_le_bytes();

assert_eq!(buffer, [5, 0]); // <- the original bytes changed
```

**When to use it:** editing a memory-mapped file or a shared buffer in place.
It needs `IntoBytes` as well, since writing through the reference means the
bytes must stay valid for the type.

#### Slices of records

`<[T]>::ref_from_bytes` views a whole buffer as a slice.

```
use zerocopy::{FromBytes, Immutable, KnownLayout, Unaligned};

#[derive(FromBytes, KnownLayout, Immutable, Unaligned, Debug, PartialEq)]
#[repr(C)]
struct Rgb { r: u8, g: u8, b: u8 }

let pixels = [255u8, 0, 0, 0, 255, 0];
let rgb = <[Rgb]>::ref_from_bytes(&pixels).unwrap();

assert_eq!(rgb.len(), 2);
assert_eq!(rgb[1], Rgb { r: 0, g: 255, b: 0 });

// The length must divide evenly.
assert!(<[Rgb]>::ref_from_bytes(&[1u8, 2, 3, 4]).is_err());
```

**When to use it:** image data, sample buffers, fixed-size record files. The
even-division check is the bounds check you would otherwise write by hand, and
getting it wrong is how a parser reads past the end.

### Copying instead

#### `read_from_bytes`

Copies the bytes into a new value, ignoring alignment entirely.

```
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, KnownLayout, Immutable, Debug, PartialEq)]
#[repr(C)]
struct Aligned { value: u32 } // <- alignment 4, so casting needs luck

// An over-aligned buffer, so offset 1 is guaranteed misaligned.
#[repr(align(4))]
struct Buffer([u8; 8]);

let buffer = Buffer([0xff, 1, 0, 0, 0, 0, 0, 0]);
let bytes = &buffer.0[..];

// Borrowing from offset 1 fails: a &u32 must be 4-aligned.
assert!(Aligned::ref_from_bytes(&bytes[1..5]).is_err());

// Copying does not care about the source address.
let value = Aligned::read_from_bytes(&bytes[1..5]).unwrap();
assert_eq!(value, Aligned { value: 1 });
```

**When to use it:** when the type has real alignment and the buffer's address is
not yours to choose. It costs one copy of the struct, which for a small header
is nothing — and it is still safer and shorter than assembling fields from
`from_le_bytes` calls.

#### `read_from_prefix`

The copying counterpart to `ref_from_prefix`.

```
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, KnownLayout, Immutable)]
#[repr(C)]
struct Magic { value: u32 }

let file = [1u8, 0, 0, 0, b'r', b'e', b's', b't'];
let (magic, rest) = Magic::read_from_prefix(&file[..]).unwrap();

assert_eq!(magic.value, 1);
assert_eq!(rest, b"rest");
```

**When to use it:** walking a format whose types are aligned, or whenever you
want an owned value rather than a borrow tied to the buffer's lifetime.

### Writing out

#### `as_bytes`

The value's bytes, borrowed.

```
use zerocopy::{Immutable, IntoBytes};

#[derive(IntoBytes, Immutable)]
#[repr(C)]
struct Message { kind: u8, code: u8 }

let msg = Message { kind: 2, code: 9 };
assert_eq!(msg.as_bytes(), &[2, 9]);
```

**When to use it:** writing to a socket or a file, hashing a struct, or
comparing two values byte-wise. It borrows, so there is no allocation — the
bytes *are* the value.

#### `write_to` and `write_to_prefix`

Writes into a buffer you provide.

```
use zerocopy::{Immutable, IntoBytes};

#[derive(IntoBytes, Immutable)]
#[repr(C)]
struct Record { a: u8, b: u8 }

let mut buffer = [0u8; 4];
Record { a: 1, b: 2 }.write_to_prefix(&mut buffer[..]).unwrap();
assert_eq!(buffer, [1, 2, 0, 0]);

// write_to requires an exact fit.
assert!(Record { a: 1, b: 2 }.write_to(&mut buffer[..]).is_err());
```

**When to use it:** assembling a packet in a reusable buffer, where allocating
per message is the cost you are avoiding. `write_to` insisting on an exact size
catches the off-by-one that `write_to_prefix` would let through.

### Endianness

#### `byteorder` types

Integers that carry their byte order, so a struct stays `Unaligned` and the wire
format is explicit.

```
use zerocopy::byteorder::{big_endian, little_endian};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct Mixed {
    network: big_endian::U16,
    local: little_endian::U16,
}

let value = Mixed {
    network: big_endian::U16::new(1),
    local: little_endian::U16::new(1),
};

assert_eq!(value.as_bytes(), &[0, 1, 1, 0]); // <- same number, opposite orders
assert_eq!(value.network.get(), 1);
```

**When to use it:** every multi-byte field in a wire or file format. They solve
two problems at once — the byte order is stated in the type rather than
remembered at each access, and the alignment stays 1 so the struct can be cast
from any buffer.

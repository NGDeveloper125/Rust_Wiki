---
title: "bytes"
version: "1.12.1"
publisher: "Carl Lerche (carllerche), Alice Ryhl (Darksonn), Core"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-23"
summary: "Byte buffers that slice and share without copying. `Bytes` clones by bumping a reference count, `BytesMut` splits a read buffer into frames, and the `Buf`/`BufMut` traits abstract over both."
categories: ["data-structures", "networking", "no-std"]
repository: "https://github.com/tokio-rs/bytes"
---

## Overview

Network code spends its life cutting buffers apart and handing the pieces
around. A read returns 8 KiB containing three and a half messages; each complete
one goes to a different handler, and the remainder waits for more data. With
`Vec<u8>` every one of those steps is a copy, and every handler that keeps its
message keeps a separate allocation.

`bytes` makes those operations pointer arithmetic on a shared allocation:

```
use bytes::Bytes;

let buffer = Bytes::from_static(b"GET /path HTTP/1.1");

// Slicing is O(1) — a new view onto the same allocation.
let method = buffer.slice(0..3);
let path = buffer.slice(4..9);

assert_eq!(method, "GET");
assert_eq!(path, "/path");

// Cloning is a reference-count bump, not a copy.
let shared = buffer.clone();
assert_eq!(shared.len(), buffer.len());
```

**Two types, with a one-way door between them.**

- **`BytesMut`** is the buffer you fill — from a socket, a file, a serialiser.
  It is mutable and can be split.
- **`Bytes`** is immutable and cheap to clone. `BytesMut::freeze` converts one
  into the other, and that direction is the normal flow: accumulate, split off a
  complete frame, freeze it, hand it on.

**The operation that makes the crate worth having is `split_to`.** It divides a
buffer in two at an index, giving each half its own handle to the same
allocation, with no copy and no new allocation. That is exactly the shape of
frame decoding, and it is why `tokio`, `hyper` and `tonic` all speak `Bytes` at
their boundaries — a payload can travel from the socket read to your handler
without being copied once.

**It is not a general replacement for `Vec<u8>`.** A buffer that is built, used
and dropped in one place gains nothing from the reference count, and `Bytes` has
no way to mutate in place. Reach for it when buffers are *shared* or *split* —
which in practice means network code and little else.

Two things to know before relying on it. `BytesMut::split_off` and friends do not
give back memory to the allocator until every handle to the underlying buffer is
dropped, so holding one small slice of a large read keeps the whole read alive —
a real source of surprising memory use in proxies. And `Bytes::from_static` is
free, taking no allocation at all, which makes it the right way to express
constants.

The crate has no required dependencies, works `no_std` with `alloc`, and needs
Rust 1.57.

## When to use it

### Use case: Splitting a stream into frames

The canonical job. Data arrives in arbitrary chunks; messages are
length-prefixed and must come out whole.

```
use bytes::{Buf, BytesMut};

/// Pull off one complete frame, or None if more data is needed.
fn next_frame(buffer: &mut BytesMut) -> Option<BytesMut> {
    if buffer.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes(buffer[..4].try_into().unwrap()) as usize;
    if buffer.len() < 4 + len {
        return None; // <- incomplete; leave it for the next read
    }
    buffer.advance(4); // <- drop the header
    Some(buffer.split_to(len))
}

let mut buffer = BytesMut::new();
buffer.extend_from_slice(&3u32.to_be_bytes());
buffer.extend_from_slice(b"abc");
buffer.extend_from_slice(&2u32.to_be_bytes());
buffer.extend_from_slice(b"hi");

assert_eq!(next_frame(&mut buffer).unwrap(), "abc");
assert_eq!(next_frame(&mut buffer).unwrap(), "hi");
assert!(next_frame(&mut buffer).is_none()); // <- buffer drained
```

**Why it fits:** `split_to` hands out the frame and leaves the remainder in
place, both pointing into the same allocation. The partial-frame case needs no
special handling — the bytes simply stay in the buffer until the next read
completes them.

### Use case: Fanning one payload out to many consumers

A message that several tasks need, without a copy apiece.

```
use bytes::Bytes;

let payload = Bytes::from(vec![0u8; 1024]);

// Each consumer gets its own handle to the same 1 KiB.
let handlers: Vec<Bytes> = (0..3).map(|_| payload.clone()).collect();

assert_eq!(handlers.len(), 3);
assert!(handlers.iter().all(|h| h.len() == 1024));

// Still one allocation: no handle is unique while the others live.
assert!(!payload.is_unique());

drop(handlers);
assert!(payload.is_unique()); // <- the last one standing
```

**Why it fits:** broadcasting to N subscribers costs N pointer copies rather
than N kilobytes. `is_unique` is the window into what is happening, and it is
also how you tell whether a buffer can be cheaply reclaimed.

### Use case: Building a response without reallocating

Writing a header and body into one buffer, then freezing it for sending.

```
use bytes::{BufMut, BytesMut};

fn build_response(body: &[u8]) -> bytes::Bytes {
    let mut out = BytesMut::with_capacity(64 + body.len());

    out.put_slice(b"HTTP/1.1 200 OK\r\nContent-Length: ");
    out.put_slice(body.len().to_string().as_bytes());
    out.put_slice(b"\r\n\r\n");
    out.put_slice(body);

    out.freeze() // <- no copy; the same allocation becomes immutable
}

let response = build_response(b"hello");
assert!(response.starts_with(b"HTTP/1.1 200 OK"));
assert!(response.ends_with(b"hello"));
```

**Why it fits:** `with_capacity` sizes the allocation once, the `put_*` methods
append without bounds checks failing, and `freeze` converts in place. The result
is one allocation for the whole response, and it is now cheaply cloneable if it
needs sending more than once.

## API map

Two buffer types and two traits. `Bytes` and `BytesMut` are the storage; `Buf`
and `BufMut` are the cursors over them, and are what generic code takes.

### `Bytes`

#### `Bytes::from_static` and `from`

Constructing an immutable buffer, with or without an allocation.

```
use bytes::Bytes;

// No allocation at all: it borrows the binary's own data.
let literal = Bytes::from_static(b"OK");
assert_eq!(literal, "OK");

// Takes ownership of a Vec without copying it.
let owned = Bytes::from(vec![1u8, 2, 3]);
assert_eq!(&owned[..], &[1, 2, 3]);

// Copies, for when you only have a borrow.
let copied = Bytes::copy_from_slice(&[4u8, 5]);
assert_eq!(copied.len(), 2);
```

**When to use it:** `from_static` for constants — protocol keywords, canned
responses — since it costs nothing. `from(Vec)` when you already own the data.
`copy_from_slice` only when you must, because it is the one that allocates.

#### `slice`

A sub-range as a new `Bytes`, sharing the allocation.

```
use bytes::Bytes;

let full = Bytes::from_static(b"key=value");
let key = full.slice(0..3);
let value = full.slice(4..);

assert_eq!(key, "key");
assert_eq!(value, "value");

// Both still point into `full`, so nothing was copied.
assert_eq!(full.len(), 9);
```

**When to use it:** carving a parsed structure out of a buffer you keep. Note
what it does *not* do — the parent allocation stays alive as long as any slice
does, so slicing four bytes out of a megabyte keeps the megabyte.

#### `split_to` and `split_off`

Divides the buffer in two, consuming the split point.

```
use bytes::Bytes;

let mut buffer = Bytes::from_static(b"headerbody");

let header = buffer.split_to(6); // <- takes the front
assert_eq!(header, "header");
assert_eq!(buffer, "body");     // <- `buffer` keeps the rest

let mut other = Bytes::from_static(b"abcdef");
let tail = other.split_off(3);  // <- takes the back
assert_eq!(other, "abc");
assert_eq!(tail, "def");
```

**When to use it:** frame decoding, where a complete message leaves the buffer
and the remainder stays. The two differ only in which half you get back, and
both are O(1) — this is the operation the crate exists for.

#### `is_unique`

Whether this handle is the only one on the allocation.

```
use bytes::Bytes;

let data = Bytes::from(vec![1u8, 2, 3]);
assert!(data.is_unique());

let shared = data.clone();
assert!(!data.is_unique());

drop(shared);
assert!(data.is_unique());
```

**When to use it:** deciding whether a buffer can be reclaimed or converted back
to `BytesMut`. `try_into_mut` uses exactly this check — it succeeds only when no
other handle exists, since mutating a shared buffer would be visible to the
others.

### `BytesMut`

#### `with_capacity` and `extend_from_slice`

Building a buffer up.

```
use bytes::BytesMut;

let mut buf = BytesMut::with_capacity(16);
assert_eq!(buf.len(), 0);
assert!(buf.capacity() >= 16);

buf.extend_from_slice(b"hello");
buf.extend_from_slice(b" world");

assert_eq!(&buf[..], b"hello world");
```

**When to use it:** whenever you know roughly the final size. Sizing up front
matters more here than for `Vec`, because a reallocation invalidates the
zero-copy property for anything already split off.

#### `freeze`

Converts to `Bytes` in place, with no copy.

```
use bytes::BytesMut;

let mut buf = BytesMut::new();
buf.extend_from_slice(b"payload");

let immutable = buf.freeze();
assert_eq!(immutable, "payload");

// Now cheaply cloneable, which BytesMut is not.
let copy = immutable.clone();
assert_eq!(copy, immutable);
```

**When to use it:** at the boundary where a buffer stops being written and
starts being shared — the end of a serialiser, the point a frame leaves the
decoder. It consumes the `BytesMut`, which is the type system recording that
nothing can mutate it any more.

#### `split` and `split_to`

The mutable equivalents, and the core of a read loop.

```
use bytes::BytesMut;

let mut buf = BytesMut::from(&b"frame1frame2"[..]);

let first = buf.split_to(6);
assert_eq!(&first[..], b"frame1");
assert_eq!(&buf[..], b"frame2");

// split() takes everything, leaving an empty buffer that keeps its capacity.
let rest = buf.split();
assert_eq!(&rest[..], b"frame2");
assert!(buf.is_empty());
```

**When to use it:** `split_to` per frame, `split` to take the whole accumulated
buffer at once. Both leave the original usable, which is what lets a read loop
keep appending into the same `BytesMut` between frames.

#### `reserve` and `unsplit`

Growing, and rejoining buffers that were split.

```
use bytes::BytesMut;

let mut buf = BytesMut::from(&b"abcdef"[..]);
let tail = buf.split_off(3);

buf.unsplit(tail); // <- cheap when the two are adjacent in one allocation
assert_eq!(&buf[..], b"abcdef");

buf.reserve(1024);
assert!(buf.capacity() >= 1024 + 6);
```

**When to use it:** `reserve` before a read, so the socket has somewhere to
write. `unsplit` when you split speculatively and the frame turned out to be
incomplete — it is O(1) if the pieces are still neighbours, and falls back to a
copy if not.

### The traits

#### `Buf`

A cursor over bytes being consumed, implemented by `Bytes`, `BytesMut`, `&[u8]`
and chains of them.

```
use bytes::Buf;

let mut data = &b"\x00\x00\x00\x2a rest"[..];

let n = data.get_u32(); // <- reads 4 bytes big-endian and advances
assert_eq!(n, 42);
assert_eq!(data, b" rest");

data.advance(1); // <- skip a byte
assert_eq!(data.remaining(), 4);
assert_eq!(data.chunk(), b"rest");
```

**When to use it:** writing a parser generic over its input. The `get_*` methods
read and advance in one step, which is what makes decoding a header read like a
sequence of statements rather than a chain of index arithmetic. They panic if
there is not enough left, so check `remaining` first on untrusted input.

#### `BufMut`

The writing counterpart.

```
use bytes::{BufMut, BytesMut};

let mut buf = BytesMut::with_capacity(16);

buf.put_u16(513);         // <- big-endian by default
buf.put_u8(7);
buf.put_slice(b"tail");

assert_eq!(&buf[..3], &[0x02, 0x01, 0x07]);
assert_eq!(&buf[3..], b"tail");
assert!(buf.remaining_mut() > 0);
```

**When to use it:** building a wire format. The `put_*` methods mirror `get_*`,
so an encoder and decoder written against the two traits read as mirror images —
which makes it much easier to see when they disagree.

#### `chain` and `take`

Treating several buffers as one, or limiting how much is read.

```
use bytes::{Buf, Bytes};

let head = Bytes::from_static(b"HTTP/1.1 200\r\n");
let body = Bytes::from_static(b"hello");

// One logical buffer over two allocations, with no copy.
let mut message = head.chain(body);
assert_eq!(message.remaining(), 19);

// take limits a Buf to the first n bytes.
let mut limited = Bytes::from_static(b"0123456789").take(4);
assert_eq!(limited.remaining(), 4);
assert_eq!(limited.copy_to_bytes(4), "0123");
```

**When to use it:** `chain` for writing a header and body without joining them —
a vectored write can take the pieces separately. `take` for honouring a
declared content length, so a parser cannot read past the end of its frame.

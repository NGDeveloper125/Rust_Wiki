---
title: "smallvec"
version: "1.16.0"
publisher: "Simon Sapin (SimonSapin), Ms2ger, cargo publish"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-08"
summary: "A `Vec` that keeps its first few elements inline instead of on the heap, spilling only when it outgrows them. A targeted allocation optimisation for collections that are usually small."
categories: ["data-structures", "performance", "no-std"]
repository: "https://github.com/servo/rust-smallvec"
---

## Overview

A `Vec` always allocates. That is the right default, and occasionally it is the
whole cost of an operation: a parser building a two-element argument list per
node, a graph walk collecting a handful of neighbours per vertex, a matcher
returning one capture. Millions of allocations, each holding two `u32`s.

`SmallVec` stores its first `N` elements inline, in the value itself, and moves
to the heap only if it grows past them:

```
use smallvec::{smallvec, SmallVec};

// Room for 4 inline. Beyond that it behaves exactly like a Vec.
let mut items: SmallVec<[u32; 4]> = smallvec![1, 2, 3];

assert!(!items.spilled());  // <- still on the stack
assert_eq!(items.len(), 3);

items.extend([4, 5, 6]);
assert!(items.spilled());   // <- outgrew the inline space, now heap-backed
assert_eq!(items.as_slice(), [1, 2, 3, 4, 5, 6]);
```

It derefs to `[T]`, so everything you do with a `Vec` — indexing, iteration,
`sort`, slicing — works unchanged.

**This is an optimisation, and it is not free.** The inline space is part of the
type, so it is paid for whether used or not:

```
use smallvec::SmallVec;

// A Vec is three words regardless of contents.
assert_eq!(std::mem::size_of::<Vec<u64>>(), 24);

// A SmallVec is that plus its inline buffer.
assert!(std::mem::size_of::<SmallVec<[u64; 8]>>() > std::mem::size_of::<Vec<u64>>());
```

So a `SmallVec<[u64; 32]>` stored in a struct makes every one of those structs
large, moves become memcpys, and a `Vec` of them uses more memory than a `Vec`
of `Vec`s would. The win only exists when the collection really is usually
small and really is created often enough to matter.

**So measure first.** The honest advice is to reach for this after a profile
shows allocation in a hot path, not before. Guessing at `N` is how you end up
with a type that is bigger *and* still spills. Servo, rustc and many parsers use
it heavily; most application code should not.

Two neighbours cover the adjacent cases. `arrayvec` has a fixed capacity and
*cannot* spill — pushing past the end is an error, which is what you want in
`no_std` with no allocator at all. `tinyvec` offers the same idea with no
`unsafe` anywhere, at the cost of requiring `T: Default`. `smallvec` sits
between them: it spills rather than failing, and uses `unsafe` internally to do
it.

The crate has no required dependencies and is `#![no_std]`. The `union` feature
saves a word per value by overlapping the inline and heap representations,
and is worth enabling when you use the type widely.

## When to use it

### Use case: A parser returning a few items per node

The archetypal case. Most nodes have one or two children; a handful have many.
With `Vec` every node allocates.

```
use smallvec::{smallvec, SmallVec};

type Args = SmallVec<[String; 2]>;

fn parse_call(input: &str) -> (String, Args) {
    let (name, rest) = input.split_once('(').unwrap_or((input, ")"));
    let args: Args = rest
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    (name.trim().to_string(), args)
}

let (name, args) = parse_call("max(a, b)");
assert_eq!(name, "max");
assert_eq!(args.len(), 2);
assert!(!args.spilled()); // <- the common case allocates nothing

let (_, many) = parse_call("sum(a, b, c, d)");
assert!(many.spilled()); // <- the rare case still works
```

**Why it fits:** the common shape is free and the uncommon one still works,
which is the property that makes this different from a fixed-size array. Note
what is *not* free here — the `String`s each allocate; only the list holding
them is inline.

### Use case: Collecting neighbours in a graph walk

A traversal that builds a short list per vertex, millions of times.

```
use smallvec::SmallVec;

// Adjacency: most vertices have few neighbours.
fn neighbours(edges: &[(u32, u32)], vertex: u32) -> SmallVec<[u32; 4]> {
    edges
        .iter()
        .filter(|(from, _)| *from == vertex)
        .map(|(_, to)| *to)
        .collect()
}

let edges = [(0, 1), (0, 2), (1, 3)];

let from_zero = neighbours(&edges, 0);
assert_eq!(&from_zero[..], [1, 2]);
assert!(!from_zero.spilled());

// Sorting, slicing and iteration work as on a Vec, via Deref.
let mut sorted = neighbours(&edges, 0);
sorted.sort_unstable_by(|a, b| b.cmp(a));
assert_eq!(&sorted[..], [2, 1]);
```

**Why it fits:** the per-vertex allocation disappears from the profile, and
nothing about the calling code changes because `SmallVec` derefs to a slice.
For `u32` neighbours the inline buffer is 16 bytes, so the type stays small
enough to pass around freely.

### Use case: Avoiding an allocation for a usually-empty result

A validator that normally finds nothing still pays for a `Vec` if it returns
one.

```
use smallvec::SmallVec;

type Errors = SmallVec<[&'static str; 2]>;

fn validate(name: &str, age: i32) -> Errors {
    let mut errors = Errors::new();
    if name.is_empty() {
        errors.push("name is empty");
    }
    if !(0..150).contains(&age) {
        errors.push("age out of range");
    }
    errors
}

let ok = validate("ada", 36);
assert!(ok.is_empty());
assert!(!ok.spilled()); // <- the happy path never touches the heap

let bad = validate("", 200);
assert_eq!(bad.len(), 2);
```

**Why it fits:** `Vec::new` does not allocate either, so the saving here is
smaller than it looks — it appears only once something is pushed. The reason to
use `SmallVec` in this shape is the one-or-two-error case, which is common
enough in validation to be worth the inline space.

## API map

`SmallVec` derefs to `[T]`, so the whole slice API is available and is not
repeated here. The entries below are what the type adds or changes relative to
`Vec`.

### Creating

#### `smallvec!`

The literal macro, mirroring `vec!`.

```
use smallvec::{smallvec, SmallVec};

let listed: SmallVec<[u8; 4]> = smallvec![1, 2, 3];
let repeated: SmallVec<[u8; 4]> = smallvec![0; 3];

assert_eq!(listed.as_slice(), [1, 2, 3]);
assert_eq!(repeated.as_slice(), [0, 0, 0]);
```

**When to use it:** wherever you would write `vec!`. The inline capacity comes
from the annotation rather than the macro, so the type has to be named
somewhere — usually a `type` alias, which keeps the capacity in one place.

#### `SmallVec::new`

An empty vector, allocating nothing.

```
use smallvec::SmallVec;

let mut v: SmallVec<[i32; 8]> = SmallVec::new();
assert!(v.is_empty());
assert!(!v.spilled());

v.push(1);
assert_eq!(v.capacity(), 8); // <- the inline capacity, not a heap one
```

**When to use it:** the default constructor. Note `capacity` reports the inline
size straight away, where `Vec::new().capacity()` is zero — nothing was
allocated, the room simply already exists.

#### `SmallVec::with_capacity`

Reserves space up front, on the heap if the request exceeds the inline size.

```
use smallvec::SmallVec;

let small: SmallVec<[u8; 8]> = SmallVec::with_capacity(4);
assert!(!small.spilled()); // <- fits inline, so no allocation

let large: SmallVec<[u8; 8]> = SmallVec::with_capacity(64);
assert!(large.spilled()); // <- allocated immediately
```

**When to use it:** when the eventual size is known and larger than the inline
buffer, to allocate once rather than growing. Asking for less than the inline
capacity is a no-op, which is the useful behaviour — it never allocates when it
does not have to.

#### `SmallVec::from_vec`

Takes ownership of an existing `Vec` without copying.

```
use smallvec::SmallVec;

// Decided by the Vec's CAPACITY, not its length.
let tight = vec![1u8, 2, 3]; // capacity 3
let inline: SmallVec<[u8; 8]> = SmallVec::from_vec(tight);
assert!(!inline.spilled()); // <- 3 <= 8, so copied inline

let mut roomy = Vec::with_capacity(64); // capacity 64
roomy.extend([1u8, 2, 3]);
let kept: SmallVec<[u8; 8]> = SmallVec::from_vec(roomy);
assert!(kept.spilled()); // <- 64 > 8, so the allocation is kept

assert_eq!(inline.as_slice(), kept.as_slice());
```

**When to use it:** converting at a boundary. The rule is worth knowing because
it is about capacity rather than length: a short `Vec` that was built with
`with_capacity(64)` stays on the heap, so a value that looks small can still be
spilled. Call `shrink_to_fit` on the `Vec` first if you want the inline path.

### Inspecting the representation

#### `spilled`

Whether the data is on the heap.

```
use smallvec::SmallVec;

let mut v: SmallVec<[u16; 2]> = SmallVec::new();
v.push(1);
v.push(2);
assert!(!v.spilled());

v.push(3); // <- one past the inline capacity
assert!(v.spilled());

// Spilling is one-way: shrinking does not move back inline.
v.pop();
v.shrink_to_fit();
assert!(!v.spilled()); // <- except shrink_to_fit does move it back
```

**When to use it:** in tests and benchmarks that assert an optimisation is
actually happening. It is the only way to confirm your chosen `N` matches
reality — if everything spills, the inline space is pure cost.

#### `inline_size`

The inline capacity, as a number.

```
use smallvec::SmallVec;

let v: SmallVec<[u8; 16]> = SmallVec::new();
assert_eq!(v.inline_size(), 16);
```

**When to use it:** in generic code over `A: Array`, and in assertions that
document the intended threshold. Rarely needed directly, since the number is
usually visible in the type.

### Converting out

#### `into_vec`

Turns it into a `Vec`, allocating only if it was still inline.

```
use smallvec::SmallVec;

let sv: SmallVec<[u8; 4]> = SmallVec::from_slice(&[1, 2, 3]);
let v: Vec<u8> = sv.into_vec();

assert_eq!(v, [1, 2, 3]);
```

**When to use it:** at an API boundary that wants a `Vec`. It is free when the
value had already spilled — the heap buffer is handed over — and copies when it
had not, which is the same cost the `Vec` would have paid anyway.

#### `into_inner`

Recovers the backing array when the vector is exactly full and inline.

```
use smallvec::SmallVec;

let full: SmallVec<[u8; 3]> = SmallVec::from_slice(&[1, 2, 3]);
assert_eq!(full.into_inner(), Ok([1, 2, 3]));

// Not exactly full, so the SmallVec comes back instead.
let partial: SmallVec<[u8; 3]> = SmallVec::from_slice(&[1, 2]);
assert!(partial.into_inner().is_err());
```

**When to use it:** when a fixed-size array is what the next step wants and you
built it incrementally. The `Result` carries the original back on failure, so
nothing is lost when the length does not match.

### Growing and shrinking

#### `push` and `insert`

The same operations as `Vec`, with a spill instead of a reallocation at the
boundary.

```
use smallvec::SmallVec;

let mut v: SmallVec<[i32; 2]> = SmallVec::new();
v.push(1);
v.insert(0, 0); // <- shifts, exactly as Vec does

assert_eq!(v.as_slice(), [0, 1]);
assert!(!v.spilled());
```

**When to use it:** as you would on a `Vec`. The only difference is where the
cost falls — the first push past `N` copies the inline elements to a new heap
allocation, so a loop that always crosses that line pays a copy a `Vec` would
not.

#### `shrink_to_fit`

Releases unused capacity, and moves back inline if the data now fits.

```
use smallvec::SmallVec;

let mut v: SmallVec<[u8; 4]> = SmallVec::from_slice(&[1, 2, 3, 4, 5]);
assert!(v.spilled());

v.truncate(2);
v.shrink_to_fit();

assert!(!v.spilled()); // <- back inline, heap buffer freed
assert_eq!(v.as_slice(), [1, 2]);
```

**When to use it:** after a large collection shrinks and will be kept around —
a cache entry, a long-lived struct field. This un-spilling is something `Vec`
has no equivalent of, and is the reason a `SmallVec` field can stay cheap across
a value's whole life.

#### `try_reserve`

Reserves capacity, returning an error instead of aborting when allocation fails.

```
use smallvec::SmallVec;

let mut v: SmallVec<[u8; 4]> = SmallVec::new();

match v.try_reserve(1024) {
    Ok(()) => assert!(v.capacity() >= 1024),
    Err(e) => eprintln!("cannot grow: {e:?}"),
}
```

**When to use it:** when the size comes from untrusted input — a length prefix
in a file or on the wire. `reserve` aborts the process on failure, which is a
poor response to a malformed header.

#### `drain` and `retain`

Removing in bulk, as on a `Vec`.

```
use smallvec::SmallVec;

let mut v: SmallVec<[u32; 8]> = (1..=6).collect();

let taken: Vec<u32> = v.drain(1..3).collect();
assert_eq!(taken, [2, 3]);
assert_eq!(v.as_slice(), [1, 4, 5, 6]);

v.retain(|n| *n % 2 == 0);
assert_eq!(v.as_slice(), [4, 6]);
```

**When to use it:** exactly as with `Vec`. Neither moves a spilled vector back
inline — the allocation is kept for reuse — so pair them with `shrink_to_fit`
when the value is long-lived and has shrunk for good.

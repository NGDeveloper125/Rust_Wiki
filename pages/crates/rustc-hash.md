---
title: "rustc-hash"
version: "2.1.3"
publisher: "rust-lang-owner"
publisher_url: "https://crates.io/users/rust-lang-owner"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-17"
summary: "The hasher `rustc` itself uses: a non-cryptographic hash tuned for speed on small integer keys, with a zero-sized `BuildHasher` and no randomness, so iteration order is reproducible."
domain: "Crypto, hashing & TLS"
categories: ["algorithms", "no-std"]
repository: "https://github.com/rust-lang/rustc-hash"
docs: "https://docs.rs/rustc-hash/2.1.3/rustc_hash/"
---

## Overview

`rustc-hash` is the hasher the Rust compiler uses on itself, published so other
programs can use it too. It is not cryptographic and does not pretend to be;
for a digest that resists forgery you want [`sha2`](sha2.md) and the
[`digest`](digest.md) traits.

Using it is an import and a `default()`:

```
use rustc_hash::{FxHashMap, FxHashSet};

let mut counts: FxHashMap<u32, u32> = FxHashMap::default();
*counts.entry(7).or_insert(0) += 1;

let mut seen: FxHashSet<&str> = FxHashSet::default();
seen.insert("a");
seen.insert("a");

assert_eq!(counts[&7], 1);
assert_eq!(seen.len(), 1);
```

Those are type aliases for `std::collections::HashMap` and `HashSet` with the
hasher substituted, so everything else about them is unchanged. Note
`default()` rather than `new()`: `new` is only defined for the standard
`RandomState`, and unlike [`foldhash`](foldhash.md) this crate ships no
extension trait to restore it.

**The defining property is that there is no randomness at all.**
`FxBuildHasher` is a unit struct — zero bytes — with a fixed algorithm and a
fixed seed. Two consequences, and which one is a feature depends entirely on
your situation:

```
use rustc_hash::FxHashMap;

// Build the same map twice, in two separate maps.
let build = || {
    let mut map: FxHashMap<u32, u32> = FxHashMap::default();
    for i in 0..8 {
        map.insert(i, i * i);
    }
    map.keys().copied().collect::<Vec<_>>()
};

// Identical iteration order — and identical on the next run of the program.
assert_eq!(build(), build());
```

The feature is **reproducibility**: a compiler that wants byte-identical output
from identical input cannot have its data structures iterating in a different
order each run, and that is precisely why `rustc` uses this. The hazard is that
it is **trivially vulnerable to collision attacks** — an attacker who knows the
algorithm, and it is public, can generate keys that all land in one bucket and
turn a map into a linked list. So: fine for compiler-internal data, keys you
generated, anything where the inputs are yours. Never for keys that came off the
network.

**Do not store the hash values.** The `Fx` name is kept for compatibility but
the algorithm behind it is not the Firefox one any more — 2.0 replaced it with a
polynomial hash designed by Orson Peters, and 2.1.1 changed it again to fix a
regression on large maps in rustc. Two changes within one major version is the
clearest possible statement that the output is not a stable value.

**Against the alternatives.** Compared to `std`'s SipHash-1-3 this is much
faster on short keys and gives up the DoS resistance that motivated SipHash.
Compared to [`foldhash`](foldhash.md) — which is, interestingly, the same
author's later design and what [`hashbrown`](hashbrown.md) now defaults to —
`rustc-hash` is deterministic where `foldhash` randomises, and `foldhash` is the
better-distributed of the two. Pick `rustc-hash` when you want reproducibility
or are matching the compiler's behaviour; pick `foldhash` when you want a fast
hash map and have no opinion beyond that.

The dependency is minimal: nothing required, `#![no_std]` at its core with the
map aliases behind the default `std` feature, MSRV 1.77, and optional `rand`
and `nightly` features.

## When to use it

### Use case: Reproducible output from a tool that uses hash maps

Where determinism is the requirement rather than a side effect.

```
use rustc_hash::FxHashMap;

// A symbol table in a code generator, emitted in iteration order.
fn emit(symbols: &[(&str, u32)]) -> String {
    let mut table: FxHashMap<&str, u32> = FxHashMap::default();
    for (name, id) in symbols {
        table.insert(name, *id);
    }

    table
        .iter()
        .map(|(name, id)| format!("{name}={id}"))
        .collect::<Vec<_>>()
        .join(";")
}

let input = [("alpha", 1), ("beta", 2), ("gamma", 3)];

// The same input gives the same output, every run.
assert_eq!(emit(&input), emit(&input));
```

**Why it fits:** a `std::collections::HashMap` seeds itself randomly per
process, so iterating one and emitting the result produces different bytes each
run — which breaks build caching, makes diffs noisy and turns an intermittent
bug into an unreproducible one. This fixes it without the usual remedy of
collecting and sorting. The honest caveat: iteration order is an implementation
detail that a `hashbrown` or crate upgrade may change, so if the output is a
guarantee rather than a convenience, sort it.

### Use case: Hot maps keyed by small integers

The case the algorithm was tuned for.

```
use rustc_hash::FxHashMap;
use std::collections::HashMap;

// Interning: map ids to data, looked up constantly.
let mut nodes: FxHashMap<u32, &str> = HashMap::with_capacity_and_hasher(
    1024,
    rustc_hash::FxBuildHasher,
);

for id in 0..1000u32 {
    nodes.insert(id, "node");
}

assert_eq!(nodes.len(), 1000);
assert_eq!(nodes.get(&500), Some(&"node"));
assert_eq!(nodes.get(&5000), None);
```

**Why it fits:** the crate's README is explicit that hashing speed on single
integers is what mattered for rustc, and that a higher-quality hash did not
reduce collisions enough to pay for itself on real workloads. That is the shape
this is for — node ids, interned string handles, type ids. Note the
`with_capacity_and_hasher` form: the alias has no `with_capacity`, because that
constructor belongs to `std`'s `RandomState`.

### Use case: A hash map in `no_std` code

Where there is no entropy source to seed from.

```
use rustc_hash::{FxBuildHasher, FxHasher};
use std::hash::{BuildHasher, Hasher};

// FxBuildHasher is a unit struct, so it needs no construction and no seed.
const HASHER: FxBuildHasher = FxBuildHasher;
let hash = HASHER.hash_one(42u32);
assert_eq!(hash, FxBuildHasher.hash_one(42u32));

// FxHasher::with_seed is const too, for a fixed non-zero seed.
const SEEDED: FxHasher = FxHasher::with_seed(99);
let _ = SEEDED;

// Zero-sized, so a map using it is smaller than a std one.
assert_eq!(std::mem::size_of::<FxBuildHasher>(), 0);
assert_eq!(
    std::mem::size_of::<std::collections::hash_map::RandomState>(),
    16
);
```

**Why it fits:** randomised hashers need entropy, and on a bare-metal target
there may be none — the crate is `#![no_std]` at its core precisely so this
works. The zero-sized `BuildHasher` is a real if small bonus: a `std` `HashMap`
carries 16 bytes of `RandomState` per map, and this carries none, which shows up
when you have many small maps. Turn off the default `std` feature and the map
aliases go away, leaving `FxHasher` and `FxBuildHasher` to use with
[`hashbrown`](hashbrown.md).

## API map

The crate is small: two aliases, two hasher types, and two optional variants
that add back the randomness the default deliberately omits.

### The default, deterministic hasher

#### `FxHashMap` and `FxHashSet`

The aliases you will use almost always.

```
use rustc_hash::{FxHashMap, FxHashSet};

// default(), not new().
let mut map: FxHashMap<&str, u32> = FxHashMap::default();
map.insert("key", 1);

let mut set: FxHashSet<u32> = FxHashSet::default();
set.insert(1);

assert_eq!(map["key"], 1);
assert!(set.contains(&1));
```

**When to use it:** as a drop-in for `HashMap` wherever the keys are yours. They
are `std`'s containers with one type parameter changed, so every method and
trait impl is the same and there is nothing new to learn — the only visible
differences are `default()` in place of `new()`, and an iteration order that no
longer varies between runs. Behind the `std` feature, so unavailable in
`no_std`.

#### `FxBuildHasher`

The zero-sized `BuildHasher` behind the aliases.

```
use rustc_hash::FxBuildHasher;
use std::collections::HashMap;
use std::hash::BuildHasher;

// Pass it to any container generic over its hasher.
let mut map: HashMap<&str, u32, FxBuildHasher> = HashMap::default();
map.insert("a", 1);

// Or hash directly; it is a unit struct, so there is nothing to construct.
let hash = FxBuildHasher.hash_one("a");
assert_eq!(hash, FxBuildHasher.hash_one("a"));
assert_eq!(map["a"], 1);
```

**When to use it:** with [`indexmap`](indexmap.md), a `hashbrown` map, or
`with_capacity_and_hasher` — anywhere you need to name the hasher rather than
use an alias, and in `no_std` where the aliases do not exist. Being a unit
struct means it is `Copy`, `Default` and free to pass around, and that a
collection using it is exactly as large as its contents.

#### `FxHasher`

The `Hasher` itself, for hashing outside a map.

```
use rustc_hash::FxHasher;
use std::hash::Hasher;

let mut hasher = FxHasher::default();
hasher.write_u32(42);
hasher.write(b"bytes");
let first = hasher.finish();

// Same writes in the same order give the same result.
let mut again = FxHasher::default();
again.write_u32(42);
again.write(b"bytes");
assert_eq!(first, again.finish());
```

**When to use it:** rarely — `BuildHasher::hash_one` is the shorter way to hash
one value, and a map builds these for you. Reach for it when you are feeding a
hash incrementally from several pieces, as a bloom filter or a checksum over
structured data would. `default()` and `with_seed()` are both `const`, so an
instance can be a `const` item.

### Adding randomness back

#### `FxRandomState` and its aliases

Per-instance random seeding, for when determinism is the wrong trade.

```
use rustc_hash::{FxHashMapRand, FxRandomState};
use std::hash::BuildHasher;

// Each instance seeds itself, so the same key hashes differently.
let first = FxRandomState::new();
let second = FxRandomState::new();
assert_ne!(first.hash_one("key"), second.hash_one("key"));

// Eight bytes, against std's sixteen.
assert_eq!(std::mem::size_of::<FxRandomState>(), 8);

let mut map: FxHashMapRand<&str, u32> = FxHashMapRand::default();
map.insert("key", 1);
assert_eq!(map["key"], 1);
```

**When to use it:** when you want Fx's speed on keys you do not fully control —
it restores the property that an attacker cannot precompute colliding keys. It
requires the `rand` feature, which pulls in a dependency the default build does
not have. If you are reaching for this, compare against
[`foldhash`](foldhash.md) first: randomised is its default, it needs no extra
feature, and it is the better-distributed design.

#### `FxSeededState`

An explicit seed you supply.

```
use rustc_hash::FxSeededState;
use std::hash::BuildHasher;

let a = FxSeededState::with_seed(7);
let b = FxSeededState::with_seed(7);
let c = FxSeededState::with_seed(8);

// Reproducible per seed, and different between seeds.
assert_eq!(a.hash_one("key"), b.hash_one("key"));
assert_ne!(a.hash_one("key"), c.hash_one("key"));
```

**When to use it:** when the seed has to come from somewhere specific — a value
in a config file, a per-tenant key, a PRNG you control so a fuzz run can be
replayed. It is the middle ground between the fixed default and full
randomisation: you get reproducibility *given the seed*, which is what a
repeatable experiment needs. `FxHashMapSeed` and `FxHashSetSeed` are the
matching aliases. The same warning applies as everywhere on this page — the
value for a given seed is not stable across crate versions.

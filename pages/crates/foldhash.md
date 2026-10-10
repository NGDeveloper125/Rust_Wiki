---
title: "foldhash"
version: "0.2.0"
publisher: "Orson Peters (orlp)"
publisher_url: "https://crates.io/users/orlp"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-10"
summary: "A fast non-cryptographic hash for hash maps and sketches, in two variants: one tuned for table lookup speed, one for statistical quality. It is `hashbrown`'s default hasher, which is why it turns up everywhere."
domain: "Crypto, hashing & TLS"
categories: ["no-std", "algorithms"]
repository: "https://github.com/orlp/foldhash"
docs: "https://docs.rs/foldhash/0.2.0/foldhash/"
---

## Overview

**`foldhash` is not a cryptographic hash, and no amount of context makes it
one.** Its own documentation says so in bold: not appropriate for any
cryptographic purpose. It is a hash for *computation* — hash maps, bloom
filters, count sketches — where the job is to scatter keys across buckets
quickly. If you need a digest someone cannot forge, you want
[`sha2`](sha2.md) and the [`digest`](digest.md) traits instead.

What it is for is making hash maps faster. `std::collections::HashMap` uses
SipHash-1-3, picked for resistance to deliberate collision attacks, and that
choice costs real time on short keys. `foldhash` takes a different position on
that trade and is the default hasher in [`hashbrown`](hashbrown.md) as of 0.15 —
which is the reason its download count is enormous relative to how often anyone
names it in a `Cargo.toml`.

Swapping it in is two lines:

```
use foldhash::{HashMap, HashMapExt};

// std's HashMap, with foldhash's BuildHasher substituted in.
let mut counts: HashMap<&str, u32> = HashMap::new();
*counts.entry("apple").or_insert(0) += 1;
*counts.entry("apple").or_insert(0) += 1;

assert_eq!(counts["apple"], 2);
```

**There are two variants, and picking the wrong one is the main way to misuse
the crate.** The `fast` module is tuned for hash tables and has, in the author's
words, known statistical imperfections — harmless when a table folds the hash
down to a bucket index, not harmless when an algorithm's correctness rests on
the bits being well distributed. The `quality` module costs a little more and is
what HyperLogLog, MinHash and similar estimators need.

Two properties to design around:

- **DoS resistance is "minimal", not absent.** `RandomState` seeds each hasher
  randomly, so an attacker cannot ship a precomputed set of colliding keys. What
  it does *not* claim to survive is an adversary who studies a long-running
  process, recovers the internal state, and generates collisions from it. If
  untrusted input becomes map keys and that is in your threat model, stay on
  SipHash.
- **The output is not stable.** Not across versions of the crate, and not across
  platforms. Never persist a `foldhash` value, put one in a file format, or send
  one over the wire — the next release is free to change it.

The dependency itself is about as light as they come: no required dependencies,
`std` behind a default feature so `no_std` works by turning it off, MSRV 1.60,
and an optional `nightly` feature that speeds up string hashing. Note the
licence is **Zlib** rather than the usual MIT/Apache-2.0 pair — permissive and
OSI-approved, but worth knowing if your project audits licences against a fixed
allowlist.

## When to use it

### Use case: Speeding up a hash map that holds trusted keys

The common case: keys you produced yourself, where SipHash is paying for a
threat you do not have.

```
use foldhash::{HashMap, HashMapExt};

#[derive(Debug, PartialEq)]
struct Metrics {
    requests: u64,
}

// Keys here are internal route names, not attacker-controlled input.
let mut by_route: HashMap<&str, Metrics> = HashMap::with_capacity(16);

by_route.insert("/health", Metrics { requests: 0 });
by_route.get_mut("/health").unwrap().requests += 3;

assert_eq!(by_route["/health"], Metrics { requests: 3 });
assert_eq!(by_route.len(), 1);
```

**Why it fits:** the import is the whole change — `foldhash::HashMap` is a type
alias for `std::collections::HashMap` with a different `BuildHasher`, so every
method, trait impl and piece of surrounding code carries on working. `HashMapExt`
exists only because `new` and `with_capacity` are defined on `HashMap<K, V,
RandomState>` for *std's* `RandomState` specifically; the trait restores them for
this one. For keys that come from the network, the honest answer is to leave
`std`'s hasher alone.

### Use case: Cardinality estimation and sketching

Algorithms whose correctness depends on the hash's distribution, not just on
hash maps working.

```
use foldhash::quality::RandomState;
use std::hash::BuildHasher;

// A one-register toy of the HyperLogLog idea: count leading zeros.
let state = RandomState::default();
let mut max_leading_zeros = 0u32;

for id in 0..10_000u32 {
    let h = state.hash_one(id);
    max_leading_zeros = max_leading_zeros.max(h.leading_zeros());
}

// With 10k well-distributed hashes, a long run of leading zeros is expected.
assert!(max_leading_zeros >= 8, "got {max_leading_zeros}");
```

**Why it fits:** an estimator like this reads individual bits of the hash and
infers a population size from them, so bias that a hash table would never notice
becomes a wrong answer. This is exactly the case the `quality` module exists for,
and the author is explicit that `fast` should not be used here. Note the
`BuildHasher` import — `hash_one` comes from that trait, not from the struct.

### Use case: Hashing with no randomness at all

`FixedState` removes the per-process seed, for when you cannot have one or do
not want one.

```
use foldhash::fast::FixedState;
use std::hash::BuildHasher;

let a = FixedState::with_seed(42);
let b = FixedState::with_seed(42);

// Same seed, same build: the same input hashes identically.
assert_eq!(a.hash_one("key"), b.hash_one("key"));

// A different seed gives a different hash.
let c = FixedState::with_seed(43);
assert_ne!(a.hash_one("key"), c.hash_one("key"));
```

**Why it fits:** a `no_std` target may have no entropy source to seed from, and a
benchmark or a differential test may want byte-identical behaviour between runs.
Both are legitimate. The two warnings are not optional, though: a fixed seed
makes HashDoS trivial, and it can turn moving entries from one map into another
into quadratic work. And "the same build" is load-bearing — a `foldhash` upgrade
may change these numbers, so do not store them or assert on literal hash values
in tests.

## API map

The crate is small: two modules offering the same three `BuildHasher` types, a
shared seed, and some `std` conveniences on top.

### Drop-in containers

#### `HashMap`, `HashSet` and their extension traits

Type aliases that pre-apply the hasher.

```
use foldhash::{HashMap, HashMapExt, HashSet, HashSetExt};

let mut map: HashMap<u32, &str> = HashMap::new();
map.insert(1, "one");

let mut set: HashSet<u32> = HashSet::with_capacity(8);
set.insert(1);
set.insert(1);

assert_eq!(map[&1], "one");
assert_eq!(set.len(), 1);
```

**When to use it:** whenever you just want faster maps and have no opinion about
the details. These are `std`'s containers, not reimplementations, so there is no
behavioural difference to learn — only iteration order changes, and that was
never guaranteed. Both aliases use `fast::RandomState`. They are behind the
`std` feature, so a `no_std` crate builds the hasher by hand instead.

#### Using the hasher with another map type

Any `HashMap`-shaped container takes a `BuildHasher`.

```
use foldhash::fast::RandomState;
use std::collections::HashMap;

// with_hasher works on std's HashMap, hashbrown's, indexmap's, and others.
let mut map: HashMap<&str, u32, RandomState> =
    HashMap::with_hasher(RandomState::default());
map.insert("key", 7);

assert_eq!(map.get("key"), Some(&7));
```

**When to use it:** with [`indexmap`](indexmap.md), a `hashbrown` map you are
configuring explicitly, or any container generic over `S`. It is also the
`no_std` route, since the type aliases are not available there. `hashbrown`
itself already defaults to this hasher, so passing it there is only needed when
you have turned its `default-hasher` feature off.

### Build hashers

#### `fast::RandomState`

The default: random per-process seed, tuned for hash tables.

```
use foldhash::fast::RandomState;
use std::hash::BuildHasher;

let first = RandomState::default();
let second = RandomState::default();

// Each instance seeds itself, so the same key hashes differently.
assert_ne!(first.hash_one("key"), second.hash_one("key"));

// Within one instance it is of course consistent.
assert_eq!(first.hash_one("key"), first.hash_one("key"));
```

**When to use it:** as the default for every hash map. The random seed is what
makes precomputed collision attacks useless, and it costs nothing at lookup
time. The struct is 8 bytes against `std`'s 16 — it holds only the per-hasher
seed and takes the larger shared seed from a global, so a map using it is
slightly smaller than the equivalent `std` one.

#### `fast::FixedState`

No randomness: the seed is a constant you choose.

```
use foldhash::fast::FixedState;
use std::collections::HashSet;
use std::hash::BuildHasher;

// Usable in const context, unlike RandomState.
const STATE: FixedState = FixedState::with_seed(0xabc);
assert_eq!(STATE.hash_one(1u32), STATE.hash_one(1u32));

let mut set: HashSet<[u8; 3], FixedState> = HashSet::with_hasher(STATE);
set.insert([1, 10, 100]);
assert!(set.contains(&[1, 10, 100]));
```

**When to use it:** `no_std` without an entropy source, and reproducible
benchmarks. `with_seed` is a `const fn`, so it works in a `static` or `const`
where `RandomState` cannot. Do not reach for it to make tests deterministic
without reading the caveat above — sort your output instead, which is robust
against a hasher change as well.

#### `fast::SeedableRandomState`

Random by default, but seedable however you like.

```
use foldhash::fast::SeedableRandomState;
use foldhash::SharedSeed;
use std::hash::BuildHasher;

let random = SeedableRandomState::random();
let fixed = SeedableRandomState::fixed();

// A fully explicit seeding, with both the per-hasher and shared parts given.
let explicit = SeedableRandomState::with_seed(99, SharedSeed::global_fixed());

assert_eq!(explicit.hash_one("x"), explicit.hash_one("x"));
assert_ne!(random.hash_one("x"), fixed.hash_one("x"));
```

**When to use it:** when the seed must come from somewhere specific — a value in
a config file, a per-tenant key, a PRNG you control for a reproducible fuzz run.
It is the only one of the three that stores an explicit `SharedSeed` reference,
so it is 16 bytes rather than 8; prefer `RandomState` unless you need the
control.

#### `quality::RandomState` and `quality::FixedState`

The same two types, tuned for distribution rather than speed.

```
use foldhash::quality::{FixedState, RandomState};
use std::hash::BuildHasher;

let random = RandomState::default();
let fixed = FixedState::with_seed(1);

// The same API as the fast module, so switching is one import.
assert_eq!(random.hash_one(7u64), random.hash_one(7u64));
assert_eq!(fixed.hash_one(7u64), fixed.hash_one(7u64));
```

**When to use it:** whenever the hash's statistical properties are part of your
algorithm — HyperLogLog, MinHash, count-min sketch, consistent hashing,
sampling by hash value. The modules are drop-in swaps for each other, so the
decision is one line and worth making deliberately rather than by default.

### Seeding and hashing directly

#### `BuildHasher::hash_one`

Hashes one value to a `u64`.

```
use foldhash::quality::RandomState;
use std::hash::BuildHasher;

let state = RandomState::default();
let hash: u64 = state.hash_one("hello world");

// Equal values hash equally; that is the only guarantee you get.
assert_eq!(hash, state.hash_one("hello world"));
```

**When to use it:** any time you want a hash rather than a map — bucketing work
across shards, a bloom filter, deciding whether to sample a request. It comes
from `std::hash::BuildHasher`, so the import is required even though the method
appears to live on the state. The value is a `u64` and nothing more: no
stability, no uniqueness, no ordering relationship to the input.

#### `SharedSeed`

The large seed table that hasher instances borrow.

```
use foldhash::SharedSeed;

// A fixed table, usable in const context.
const SEED: &SharedSeed = SharedSeed::global_fixed();

// And one derived from a single u64.
let derived = SharedSeed::from_u64(0x1234_5678);

assert!(core::ptr::eq(SEED, SharedSeed::global_fixed()));
let _ = derived;
```

**When to use it:** almost never directly — `RandomState` and `FixedState` each
reference the right global for you, which is the whole reason they fit in 8
bytes. You need it only when constructing a `SeedableRandomState` or a
`FoldHasher` by hand, where the shared part has to be named explicitly.
`global_random` is the randomised counterpart to `global_fixed`.

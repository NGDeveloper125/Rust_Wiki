---
title: "hashbrown"
version: "0.17.1"
publisher: "Amanieu d'Antras (Amanieu), rust-lang-owner"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-08"
summary: "The SwissTable hash map that `std::collections::HashMap` is built from, available as a crate — for `no_std` code, custom allocators, the low-level `HashTable`, and lookups that don't allocate a key."
categories: ["data-structures", "collections", "no-std"]
repository: "https://github.com/rust-lang/hashbrown"
---

## Overview

`hashbrown` is a Rust port of Google's SwissTable, and the thing worth knowing
first is that you are almost certainly already using it: since Rust 1.36,
[`std::collections::HashMap`](../concepts/collections-strings/hashmap-and-hashset.md)
*is* hashbrown, vendored into the standard library. So the question this page
answers is not "is it fast?" — it's the same code either way — but "what do I
get by depending on it directly, that `std` doesn't hand me?"

Four things, and if you need none of them, use `std`:

- **`no_std`.** The crate is `#![no_std]` and needs only `alloc`, so a firmware,
  kernel or WASM crate that can't link `std` still gets a hash map.
- **A faster default hasher.** `std` hashes with SipHash-1-3, chosen to resist
  hash-flooding from untrusted input. hashbrown defaults to
  [foldhash](https://crates.io/crates/foldhash), which is substantially faster
  and, in its own words, *minimally* DoS-resistant: it seeds itself randomly per
  instance, but its authors explicitly don't stand behind it against someone who
  can study a long-running process, recover that state and craft collisions. The
  right trade for a compiler's symbol table; worth a second thought for a map
  keyed by whatever an HTTP request sent you.
- **`HashTable`.** A lower-level table where *you* supply the hash and the
  equality closure, and the table stores neither a key nor a hasher. It's what
  you want for interners, caches keyed by a derived value, and anything where
  the "key" is already inside the element.
- **API that `std` hasn't stabilised.** `entry_ref`, `try_insert`,
  `allocation_size`, `extract_if`, and construction in a custom allocator via
  `new_in`.

The costs are small but real: one more dependency to audit and keep current, a
type that isn't `std::collections::HashMap` (so it doesn't cross API boundaries
that name that type), and — if you take the default hasher — an explicit
decision about untrusted input. It requires Rust 1.85 and is maintained under
the rust-lang organisation by the same author as `std`'s copy, so it is about as
low-risk as a third-party dependency gets.

The default feature set (`default-hasher`, `inline-more`, `allocator-api2`,
`equivalent`, `raw-entry`) is what most callers want. Turn off `default-hasher`
if you're supplying your own `BuildHasher` and don't want `foldhash` in the tree
at all.

## When to use it

The three situations below are where depending on hashbrown directly earns the
extra line in `Cargo.toml`. Outside them, `std::collections::HashMap` is the
same data structure with one less dependency.

### Use case: A hash map in a `no_std` crate

Firmware, a kernel module, or a WASM target built without `std` still needs to
associate keys with values, and `std::collections::HashMap` isn't reachable.
hashbrown is `#![no_std]` and needs only `alloc`, so it drops straight in.

```
#![no_std]

extern crate alloc; // <- hashbrown needs an allocator, not std

use hashbrown::HashMap;

/// The latest reading per sensor id, on a target with a heap but no `std`.
pub fn latest(readings: &[(u16, i32)]) -> HashMap<u16, i32> {
    let mut latest = HashMap::new();
    for &(sensor, value) in readings {
        latest.insert(sensor, value); // <- last reading for a sensor wins
    }
    latest
}
```

**Why it fits:** it is the same table `std` uses, so nothing about performance
or behaviour changes when a crate later gains `std` support — only the import
does.

### Use case: Counting `&str` keys in a `HashMap<String, _>`

Owning the key means `String`, but every lookup arrives as `&str`. With the
normal `entry` API you must hand over an owned `String` before you know whether
the key is even present, so a hot loop over repeated words allocates on every
iteration and throws most of them away. `entry_ref` takes the borrowed form and
only calls `to_owned` when it actually inserts.

```
use hashbrown::HashMap;

fn word_counts(words: &[&str]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in words {
        // Allocates a String only the first time each word is seen.
        *counts.entry_ref(*word).or_insert(0) += 1;
    }
    counts
}

let counts = word_counts(&["swiss", "table", "swiss"]);
assert_eq!(counts["swiss"], 2);
assert_eq!(counts.len(), 2);
```

**Why it fits:** the allocation count drops from one per lookup to one per
distinct key, without giving up the ergonomics of `entry`. `std` has no
stable equivalent.

### Use case: An interner, where the key is the value

A string interner maps content to an id, but storing the content twice — once
as the key, once in the arena — is exactly what you were trying to avoid.
`HashTable` stores only the element and asks you for the hash and the equality
test, so the table can hold ids while comparing the strings they point at.

```
use hashbrown::HashTable;
use std::hash::{BuildHasher, RandomState};

struct Interner {
    strings: Vec<String>,
    table: HashTable<u32>, // <- ids only; the text lives in `strings`
    hasher: RandomState,
}

impl Interner {
    fn intern(&mut self, text: &str) -> u32 {
        let hash = self.hasher.hash_one(text);
        let eq = |id: &u32| self.strings[*id as usize] == text;
        if let Some(&id) = self.table.find(hash, eq) {
            return id;
        }
        let id = self.strings.len() as u32;
        self.strings.push(text.to_owned());
        let strings = &self.strings;
        let hasher = &self.hasher;
        self.table
            .insert_unique(hash, id, |id| hasher.hash_one(&strings[*id as usize]));
        id
    }
}

let mut interner = Interner {
    strings: Vec::new(),
    table: HashTable::new(),
    hasher: RandomState::new(),
};
assert_eq!(interner.intern("swisstable"), 0);
assert_eq!(interner.intern("swisstable"), 0); // <- same id, no second copy
```

**Why it fits:** no `HashMap` can express "the key is derived from somewhere
else", because it insists on storing one. `HashTable` is the escape hatch, and
it's safe code — the genuinely `unsafe` raw API sits below it.

## API map

`HashMap` and `HashSet` mirror their `std` counterparts closely enough that the
[std collections page](../concepts/collections-strings/hashmap-and-hashset.md)
carries over; the entries below concentrate on constructing them, on the parts
that have no stable `std` equivalent, and on `HashTable`.

### Creating a map

Every constructor comes in an `_in` variant that takes an allocator, and a
`with_hasher` variant that takes a `BuildHasher`. Only the common ones are
listed.

#### `HashMap::new`

An empty map with the default hasher and no allocation until the first insert.
Requires the `default-hasher` feature, which is on by default.

```
use hashbrown::HashMap;

let mut map: HashMap<&str, u32> = HashMap::new();
map.insert("swisstable", 1);
assert_eq!(map.len(), 1);
```

**When to use it:** the default for internal maps whose keys you control. Where
the keys come from untrusted input and an attacker could probe the process,
build with `with_hasher` and a hasher that targets that threat — `std`'s
`RandomState` (SipHash-1-3) being the obvious one.

#### `HashMap::with_capacity`

Pre-allocates room for at least `capacity` elements, so filling the map does no
rehashing.

```
use hashbrown::HashMap;

let mut map: HashMap<u32, u32> = HashMap::with_capacity(1024);
map.insert(1, 1);
assert!(map.capacity() >= 1024);
```

**When to use it:** when you know roughly how many entries you're about to
insert. Guessing high wastes memory that `shrink_to_fit` can reclaim; guessing
low just costs the rehashes you were avoiding.

#### `HashMap::with_hasher`

Builds a map over a specific `BuildHasher`. It is a `const fn`, so it also
serves for a map in a `static`.

```
use hashbrown::HashMap;
use std::hash::RandomState;

// SipHash-1-3, the same hasher std::collections::HashMap uses.
let mut map: HashMap<String, u32, RandomState> = HashMap::with_hasher(RandomState::new());
map.insert("from-the-network".to_owned(), 1);
assert_eq!(map.len(), 1);
```

**When to use it:** whenever the map's keys can be chosen by someone else, or
when you want a specific hasher (`rustc-hash` for small integer keys, a
fixed-seed hasher for reproducible iteration in tests).

### Reading and writing

#### `HashMap::insert`

Inserts a pair, returning the previous value if the key was already present.
The stored key is *not* replaced on overwrite.

```
use hashbrown::HashMap;

let mut map = HashMap::new();
assert_eq!(map.insert("k", 1), None);
assert_eq!(map.insert("k", 2), Some(1)); // <- returns the value it displaced
```

**When to use it:** the default write. When you need to know whether you're
overwriting *before* building the value, use `entry`; when overwriting is a bug,
use `try_insert`.

#### `HashMap::get`

Looks a key up by any borrowed form of it, returning `Option<&V>`.

```
use hashbrown::HashMap;

let mut map: HashMap<String, u32> = HashMap::new();
map.insert("swisstable".to_owned(), 7);
assert_eq!(map.get("swisstable"), Some(&7)); // <- &str against a String key
assert_eq!(map.get("absent"), None);
```

**When to use it:** any read where a missing key is normal. Reach for
`get_key_value` when you need the stored key too — useful when the key carries
more than what it compares on.

#### `HashMap::get_mut`

The same lookup, handing back a mutable reference.

```
use hashbrown::HashMap;

let mut map = HashMap::new();
map.insert("hits", 1u32);
if let Some(hits) = map.get_mut("hits") {
    *hits += 1;
}
assert_eq!(map["hits"], 2);
```

**When to use it:** to modify a value you know is there. If it might be absent
and you'd insert a default, `entry(..).or_insert(..)` does both in one lookup
rather than two.

#### `HashMap::remove`

Removes a key, returning its value.

```
use hashbrown::HashMap;

let mut map = HashMap::new();
map.insert("temp", 42);
assert_eq!(map.remove("temp"), Some(42));
assert_eq!(map.remove("temp"), None);
```

**When to use it:** when you want the value back. `remove_entry` returns the key
as well, which matters when the key is an owned allocation you want to reuse.

#### `HashMap::try_insert`

Inserts only if the key is vacant. On collision it returns an `OccupiedError`
carrying both the existing entry and the value you failed to insert, so nothing
is dropped silently.

```
use hashbrown::HashMap;

let mut map = HashMap::new();
assert!(map.try_insert("id", 1).is_ok());

match map.try_insert("id", 2) {
    Ok(_) => unreachable!(),
    Err(err) => {
        assert_eq!(*err.entry.get(), 1); // <- what was already there
        assert_eq!(err.value, 2);        // <- what didn't go in
    }
}
```

**When to use it:** when a duplicate key means a bug — parsing a config with
unique names, registering handlers — and you want to report it rather than
overwrite. `insert` silently discards the old value, which hides exactly this
class of mistake.

#### `HashMap::get_disjoint_mut`

Takes an array of `N` keys and returns `N` independent mutable references,
checking at runtime that the keys are distinct.

```
use hashbrown::HashMap;

let mut balances = HashMap::new();
balances.insert("alice", 100i64);
balances.insert("bob", 50);

if let [Some(from), Some(to)] = balances.get_disjoint_mut(["alice", "bob"]) {
    *from -= 25;
    *to += 25;
}
assert_eq!(balances["alice"], 75);
```

**When to use it:** when you need to mutate two entries at once and the borrow
checker won't let you call `get_mut` twice. It panics if the keys overlap, so
pass keys you've already established are distinct.

### Entry APIs

The entry APIs do one lookup where the get-then-insert pattern does two, and
they're the only way to build a value that depends on whether the key was
already there.

#### `HashMap::entry`

Returns an `Entry` for a key, taking ownership of the key up front.

```
use hashbrown::HashMap;

let mut totals: HashMap<&str, u32> = HashMap::new();
for (name, amount) in [("a", 3), ("b", 4), ("a", 5)] {
    *totals.entry(name).or_insert(0) += amount;
}
assert_eq!(totals["a"], 8);
```

**When to use it:** the workhorse for accumulating into a map. When the key type
is an owned allocation and you're holding a borrow of it, `entry_ref` avoids
paying for the clone on every miss-free lookup.

#### `HashMap::entry_ref`

Like `entry`, but keyed by a borrowed form. The owned key is created with
`to_owned` only in the vacant case.

```
use hashbrown::HashMap;

let mut seen: HashMap<String, u32> = HashMap::new();
let key: &str = "swisstable";

*seen.entry_ref(key).or_insert(0) += 1; // <- allocates the String
*seen.entry_ref(key).or_insert(0) += 1; // <- does not
assert_eq!(seen[key], 2);
```

**When to use it:** any `HashMap<String, _>` or `HashMap<PathBuf, _>` written in
a loop over borrowed keys. This has no stable `std` equivalent and is one of the
better reasons to depend on hashbrown directly.

#### `Entry::or_insert_with`

Supplies the default lazily, so an expensive value is only built when the key
is actually absent.

```
use hashbrown::HashMap;

let mut cache: HashMap<u32, Vec<u8>> = HashMap::new();
let entry = cache.entry(7).or_insert_with(|| {
    vec![0; 4096] // <- only allocated on a miss
});
assert_eq!(entry.len(), 4096);
```

**When to use it:** whenever the default costs anything to construct. Prefer
`or_insert` for a plain value like `0` — the closure buys nothing there —
and `or_insert_with_key` when the default is derived from the key.

#### `Entry::and_modify`

Applies a function to the value if it exists, then chains into an insert if it
doesn't.

```
use hashbrown::HashMap;

let mut retries: HashMap<&str, u32> = HashMap::new();
for _ in 0..3 {
    retries.entry("job").and_modify(|n| *n += 1).or_insert(1);
}
assert_eq!(retries["job"], 3);
```

**When to use it:** when the update and the initial value differ — incrementing
an existing counter but starting a new one at `1`. If both are the same
expression, `*entry.or_insert(0) += 1` reads better.

### The low-level table

`HashTable<T>` stores bare elements. It holds no hasher and no key, so every
operation that has to hash or compare takes a closure from you. That is the
whole point: the "key" can live anywhere, including inside `T` or in a separate
arena.

#### `HashTable::new`

An empty table. `const fn`, and it allocates nothing until the first insert.

```
use hashbrown::HashTable;

let table: HashTable<u64> = HashTable::new();
assert!(table.is_empty());
```

**When to use it:** as the backing store for an interner, a cache keyed by a
derived value, or any map whose key is already part of the element. If you'd
have to store the key twice with a `HashMap`, this is the fix.

#### `HashTable::insert_unique`

Inserts an element you have already established is absent, given its hash and a
closure that can rehash any element during a resize.

```
use hashbrown::HashTable;
use std::hash::{BuildHasher, RandomState};

let hasher = RandomState::new();
let mut table: HashTable<String> = HashTable::new();

let value = String::from("swisstable");
let hash = hasher.hash_one(&value);
table.insert_unique(hash, value, |v| hasher.hash_one(v)); // <- rehash on grow
assert_eq!(table.len(), 1);
```

**When to use it:** after a `find` that came back empty, or when the data is
known-unique (deserialising a set you built yourself). It does not check for
duplicates — inserting one twice leaves two copies that only `find` will ever
disagree about.

#### `HashTable::find`

Looks up by hash plus an equality closure.

```
use hashbrown::HashTable;
use std::hash::{BuildHasher, RandomState};

let hasher = RandomState::new();
let mut table: HashTable<String> = HashTable::new();
let value = String::from("swisstable");
let hash = hasher.hash_one(&value);
table.insert_unique(hash, value, |v| hasher.hash_one(v));

let found = table.find(hasher.hash_one("swisstable"), |v| v == "swisstable");
assert_eq!(found.map(String::as_str), Some("swisstable"));
```

**When to use it:** the read side of any `HashTable`. The hash you pass must
come from the same hasher you inserted with, or the lookup silently misses —
this is the mistake to watch for, because nothing type-checks it.

#### `HashTable::entry`

The table's entry API: hash, equality closure, and a rehash closure, returning
an `Entry` you can fill.

```
use hashbrown::HashTable;
use std::hash::{BuildHasher, RandomState};

let hasher = RandomState::new();
let mut table: HashTable<(u32, u32)> = HashTable::new();

// Count occurrences of 7, keyed on the first tuple field only.
let entry = table
    .entry(hasher.hash_one(7u32), |(k, _)| *k == 7, |(k, _)| hasher.hash_one(k))
    .or_insert((7, 0));
entry.into_mut().1 += 1;
assert_eq!(table.len(), 1);
```

**When to use it:** find-or-insert against a table, in one lookup. The three
closures are verbose enough that a `HashMap` is better whenever you can
actually store the key.

#### `HashTable::find_entry`

Finds an element and hands back an `OccupiedEntry` you can remove through —
`Err` carries the table back when there's no match, so nothing is borrowed away
on a miss.

```
use hashbrown::HashTable;
use std::hash::{BuildHasher, RandomState};

let hasher = RandomState::new();
let mut table: HashTable<String> = HashTable::new();
let value = String::from("stale");
let hash = hasher.hash_one(&value);
table.insert_unique(hash, value, |v| hasher.hash_one(v));

if let Ok(entry) = table.find_entry(hash, |v| v == "stale") {
    let (removed, _) = entry.remove();
    assert_eq!(removed, "stale");
}
assert!(table.is_empty());
```

**When to use it:** removing by lookup, or reading and then deciding to remove.
`find` alone can't remove, because it only yields a reference.

### Sets and key equivalence

#### `HashSet`

The same table with `()` values, and the same relationship to `std`: identical
structure, plus `no_std` and hashbrown's extra methods.

```
use hashbrown::HashSet;

let a: HashSet<u32> = [1, 2, 3].into_iter().collect();
let b: HashSet<u32> = [2, 3, 4].into_iter().collect();

let mut common: Vec<_> = a.intersection(&b).copied().collect();
common.sort_unstable();
assert_eq!(common, vec![2, 3]);
```

**When to use it:** membership tests and set algebra, exactly as with `std`'s.
If you need the elements in order, `BTreeSet` is the one to reach for instead.

#### `Equivalent`

The trait that decides whether a lookup key matches a stored key. It's blanket
implemented via `Borrow`, so `&str` finds `String` keys for free — implement it
by hand when the relationship isn't a plain borrow.

```
use hashbrown::{Equivalent, HashMap};

#[derive(Hash, PartialEq, Eq)]
struct Id(String, u32);

// Look Id up by its name alone.
struct ByName<'a>(&'a str);

impl std::hash::Hash for ByName<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl Equivalent<String> for ByName<'_> {
    fn equivalent(&self, key: &String) -> bool {
        key == self.0
    }
}

let mut map: HashMap<String, u32> = HashMap::new();
map.insert("swisstable".to_owned(), 1);
assert_eq!(map.get(&ByName("swisstable")), Some(&1));
```

**When to use it:** when the natural lookup key isn't a `Borrow` of the stored
key — a tuple key you want to search by one field, or a wrapper with different
`Eq` semantics. Whatever you implement must hash identically to the stored key,
or the lookup misses.

### Capacity and memory

#### `HashMap::try_reserve`

Reserves capacity, returning `Err` instead of aborting when the allocation
fails.

```
use hashbrown::HashMap;

let mut map: HashMap<u32, u32> = HashMap::new();
match map.try_reserve(64) {
    Ok(()) => assert!(map.capacity() >= 64),
    Err(err) => eprintln!("cannot size the map: {err}"),
}
```

**When to use it:** when the capacity is derived from input you don't control —
a length prefix in a file or on the wire. `reserve` aborts the process on OOM,
which is a poor answer to a malformed header.

#### `HashMap::allocation_size`

The bytes the table's allocation occupies, excluding anything the keys and
values own.

```
use hashbrown::HashMap;

let mut map: HashMap<u64, u64> = HashMap::with_capacity(100);
map.insert(1, 1);
assert!(map.allocation_size() > 0);
```

**When to use it:** accounting for memory in a cache with a byte budget. Note
what it excludes — for `HashMap<String, Vec<u8>>` the heap the entries own
dwarfs the table, and you have to add it up yourself.

#### `HashMap::extract_if`

Removes and yields the entries matching a predicate, leaving the rest.

```
use hashbrown::HashMap;

let mut sessions: HashMap<u32, u64> = [(1, 100), (2, 900), (3, 200)].into_iter().collect();
let expired: Vec<_> = sessions.extract_if(|_, last_seen| *last_seen < 500).collect();

assert_eq!(expired.len(), 2);
assert_eq!(sessions.len(), 1); // <- only session 2 survives
```

**When to use it:** when you need the removed entries — expiring cache items you
then have to close, flush or log. `retain` is simpler when you only want them
gone.

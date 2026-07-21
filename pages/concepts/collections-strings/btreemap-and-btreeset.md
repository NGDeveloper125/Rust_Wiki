---
title: "BTreeMap & BTreeSet"
area: "Collections & Strings"
embedded_support: partial
groups: ["Collections & Strings", "Working with Collections", "Collections"]
related_syntax: ["[ ]", ".", ".. / ..= / ..."]
see_also: ["HashMap & HashSet", "Vec<T>", "Slices"]
---

## Explanation

`BTreeMap<K, V>` stores key-value pairs the same way
[`HashMap`](hashmap-and-hashset.md) does, but keeps its keys in sorted
order at all times, backed by a B-tree rather than a hash table.
`BTreeSet<T>` is the ordered counterpart to `HashSet<T>` — a
`BTreeSet<T>` is, internally, a `BTreeMap<T, ()>`. Both require their key
type to implement `Ord` (a total ordering) rather than `Hash`+`Eq`,
since ordering, not hashing, is what places a key in the tree.

Because a B-tree keeps each node holding several keys in a small,
cache-friendly array rather than a single key per node (as a plain
binary search tree would), lookups, inserts, and removes are all
O(log n) — slightly slower per-operation than a `HashMap`'s O(1)
average, but with a guarantee `HashMap` can't offer: iterating a
`BTreeMap`/`BTreeSet` always visits keys in ascending sorted order, with
no extra sorting step required.

That sorted-order guarantee is what `BTreeMap`/`BTreeSet` are for.
Beyond plain iteration, `.range(lo..hi)` walks only the keys inside a
given bound in sorted order — a query a `HashMap` simply cannot answer
without collecting and sorting everything first. Anywhere the data needs
to come back in order, or a "give me everything between X and Y" query
is part of the problem, that's the signal to reach for the `BTree`
variant over the `Hash` one.

The choice between the two families is almost always about whether
order is part of the requirement: pick `HashMap`/`HashSet` by default
for pure key lookup or membership, and switch to `BTreeMap`/`BTreeSet`
the moment sorted iteration or range queries enter the picture.

## Basic usage example

```
use std::collections::BTreeMap;

let mut scores: BTreeMap<u32, &str> = BTreeMap::new();
scores.insert(95, "Priya"); // <- kept in sorted-by-key order automatically
scores.insert(42, "Sam");
scores.insert(71, "Jordan");

for (score, name) in &scores { // <- always iterates in ascending key order
    println!("{score}: {name}");
}
```

**Restriction:** the key type must implement `Ord` (a total ordering) —
a type with only a partial ordering, like `f64` (`NaN` compares to
nothing), can't be used as a `BTreeMap`/`BTreeSet` key directly without
wrapping it in a type that provides a total order.

## Best practices & deeper information

### Scenario: Working with collections

A leaderboard or a time-ordered log is exactly the shape `BTreeMap` is
for: inserts land in sorted position automatically, and `.range()`
answers "everything between these two bounds" without a separate sort
step.

```
use std::collections::BTreeMap;

let mut sensor_log: BTreeMap<u64, f64> = BTreeMap::new(); // keyed by unix-timestamp
sensor_log.insert(1_700_000_030, 21.4);
sensor_log.insert(1_700_000_010, 20.9);
sensor_log.insert(1_700_000_020, 21.1);

for (timestamp, reading) in sensor_log.range(1_700_000_015..1_700_000_031) { // <- ordered range query, no sorting needed
    println!("{timestamp}: {reading}");
}
```

**Why this way:** the
[std docs](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.range)
note that `range` is the reason to reach for `BTreeMap` over `HashMap`
whenever a bounded, ordered subset of the keys is needed — `HashMap` has
no equivalent operation.

### Scenario: Implementing traits

A struct used as a `BTreeMap`/`BTreeSet` key needs `Ord` (plus
`PartialOrd`, `Eq`, `PartialEq`), and deriving all four orders fields
lexicographically in declaration order — put the primary sort key first.

```
use std::collections::BTreeSet;

#[derive(PartialEq, Eq, PartialOrd, Ord)] // <- Ord is what BTreeSet requires; fields compare in this order
struct RankedPlayer {
    score: u32,   // primary: higher score first requires reversing at insert/read time
    name: String, // tie-breaker: alphabetical when scores match
}

let mut leaderboard: BTreeSet<RankedPlayer> = BTreeSet::new();
leaderboard.insert(RankedPlayer { score: 95, name: "Priya".into() });
leaderboard.insert(RankedPlayer { score: 95, name: "Alex".into() });
```

**Why this way:** deriving `Ord` on a struct compares fields in
declaration order until one differs — the
[std docs](https://doc.rust-lang.org/std/cmp/trait.Ord.html#derivable)
document this lexicographic behavior, which is why the tie-breaking
field belongs after the primary sort key, not before it.

## Embedded Rust Notes

**Partial support.** Unlike `HashMap`/`HashSet`, `BTreeMap`/`BTreeSet`
*are* part of the `alloc` crate (`alloc::collections::BTreeMap`) — since
ordering only needs `Ord`, not a hasher or a source of randomness, they
carry none of the std-only baggage that keeps `HashMap` out of `alloc`.
That makes them usable in `#![no_std]` code the moment a
`#[global_allocator]` is configured, with no extra crate needed the way
`hashbrown` is needed for hash-based lookups. They still require a heap,
though — on allocator-free targets, a fixed-capacity, sorted
`heapless`-style structure or a plain sorted array with binary search is
the usual substitute.

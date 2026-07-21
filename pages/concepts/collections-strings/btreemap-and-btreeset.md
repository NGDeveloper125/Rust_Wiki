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

## Explanation (Embedded)

Unlike `HashMap`/`HashSet`, `BTreeMap`/`BTreeSet` *are* part of `alloc`
(`alloc::collections::BTreeMap`) — ordering only needs `Ord`, not a
hasher or a source of randomness, so they carry none of the std-only
baggage that keeps `HashMap` out of `alloc`. The moment `extern crate
alloc` plus a `#[global_allocator]` are in place, `alloc::collections::
BTreeMap` behaves exactly like `std`'s version, sorted iteration and
`.range()` included, with no extra crate needed the way `hashbrown` is
needed for hash-based lookups.

It's worth being direct, though: of this batch of collection pages, this
is the least central embedded story. The sorted-order/range-query
use case this page's Explanation builds around — leaderboards,
time-ordered logs, "everything between X and Y" — shows up less often in
typical firmware, where the data sets tend to be small and either fixed
at compile time (a calibration table) or better served by a
`heapless`-style fixed-capacity map that drops the ordering guarantee
entirely because nothing downstream needs it. And `BTreeMap` still needs
a heap regardless — for a data set that's genuinely fixed and known
ahead of time, paying for an allocator at all is often unnecessary: a
`const` sorted array searched with `.binary_search_by_key()` gives the
same ordered lookup `BTreeMap` would, using only `core`.

## Basic usage example (Embedded)

```
extern crate alloc;
use alloc::collections::BTreeMap;

fn build_sensor_log() -> BTreeMap<u64, f32> {
    let mut log = BTreeMap::new(); // <- behaves exactly like std's BTreeMap once alloc + a #[global_allocator] are configured
    log.insert(1_700_000_010, 20.9);
    log.insert(1_700_000_030, 21.4);
    log
}
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

A small set of calibration points that never changes at runtime is a
case where reaching for `BTreeMap` costs a heap allocation for data that
was already known at compile time; a `const` sorted array searched with
`.binary_search_by_key()` gives the same ordered lookup without ever
needing an allocator.

```
use alloc::collections::BTreeMap; // AVOID for a fixed, compile-time-known point set: needs alloc + a configured allocator

fn calibration_lookup_alloc() -> BTreeMap<u16, f32> { // <- BTreeMap: correct, but pays for a heap allocation for 4 fixed points
    let mut points = BTreeMap::new();
    points.insert(0, -40.0);
    points.insert(512, 0.0);
    points.insert(1024, 20.0);
    points.insert(2047, 85.0);
    points
}

// PREFER on a no-heap (or heap-avoiding) target: the same points as a sorted const array
const CALIBRATION_POINTS: [(u16, f32); 4] = [
    (0, -40.0),
    (512, 0.0),
    (1024, 20.0),
    (2047, 85.0),
];

fn calibration_lookup_array(raw: u16) -> Option<f32> {
    CALIBRATION_POINTS
        .binary_search_by_key(&raw, |&(k, _)| k) // <- binary_search_by_key: the ordered lookup BTreeMap.get() gives, no heap
        .ok()
        .map(|i| CALIBRATION_POINTS[i].1)
}
```

**Why this way:** `BTreeMap`'s ordering guarantee is only worth its
heap allocation when the key set genuinely changes at runtime; for a
fixed point set known at compile time, a sorted array plus binary search
gives the same O(log n) ordered lookup through `core` alone, with
nothing to allocate at all.

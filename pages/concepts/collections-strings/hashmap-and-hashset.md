---
title: "HashMap & HashSet"
area: "Collections & Strings"
embedded_support: partial
groups: ["Collections & Strings", "Working with Collections", "Collections"]
related_syntax: ["[ ]", "."]
see_also: ["BTreeMap & BTreeSet", "Vec<T>", "Slices"]
---

## Explanation

`HashMap<K, V>` stores key-value pairs and answers "what value is this
key associated with?" in average O(1) time by hashing the key rather
than scanning for it. `HashSet<T>` is the same idea with no value
attached — it's implemented internally as a `HashMap<T, ()>` — and
answers "have I seen this value before?" It's the natural structure for
deduplicating a sequence or testing membership without a linear scan.

Both require their key type to implement `Eq` and `Hash`: `Eq` so two
keys can be compared for equality, and `Hash` so a key can be reduced to
the bucket it belongs in. For a struct or enum used as a key, `#[derive(PartialEq, Eq, Hash)]`
is almost always sufficient, as long as every field also implements
those traits and equal values are guaranteed to hash identically. By
default, both types use `SipHash`, a hash function chosen specifically
to resist denial-of-service attacks where an adversary crafts inputs
that all collide into the same bucket — a real concern for, say, a map
keyed by attacker-controlled HTTP header names.

The trade-off for that O(1) average lookup is that iteration order is
unspecified and can change between runs of the same program, even with
the same insertions — `HashMap`/`HashSet` make no promise about the
order elements come back in. When a stable, sorted iteration order is
part of the requirement — a leaderboard, a time-ordered log, anything
where "in order" matters — [`BTreeMap`/`BTreeSet`](btreemap-and-btreeset.md)
trade a bit of raw lookup speed for exactly that guarantee.

Reach for `HashMap`/`HashSet` whenever the problem is fundamentally "look
this up by key" or "is this here" and ordering genuinely doesn't matter
— they're the right default for that shape of problem, the same way
[`Vec<T>`](vec.md) is the right default for an ordered sequence.

## Basic usage example

```
use std::collections::HashMap;

let mut inventory: HashMap<&str, u32> = HashMap::new();
inventory.insert("widget", 42); // <- key-value pair stored; O(1) average insert
inventory.insert("gadget", 17);

println!("{:?}", inventory.get("widget")); // Some(42)
```

**Restriction:** iteration order over a `HashMap`/`HashSet` is
unspecified and not guaranteed to stay stable across insertions or
program runs — never rely on it; use `BTreeMap`/`BTreeSet` when order
matters.

## Best practices & deeper information

### Scenario: Working with collections

The entry API is the idiomatic way to insert-or-update in a single
lookup, and a `HashSet` is the idiomatic way to deduplicate a sequence
without writing a manual "have I seen this" loop.

```
use std::collections::{HashMap, HashSet};

let log = ["order_placed", "order_shipped", "order_placed", "order_cancelled", "order_placed"];

let mut counts: HashMap<&str, u32> = HashMap::new();
for event in log {
    *counts.entry(event).or_insert(0) += 1; // <- one lookup does both "insert if missing" and "update"
}

let unique_events: HashSet<&str> = log.into_iter().collect(); // <- HashSet collapses duplicates automatically
println!("{:?} unique event kinds", unique_events.len());
```

**Why this way:** `entry` avoids the double lookup of checking
`contains_key` and then inserting separately, which the
[std docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry)
present as the idiomatic pattern for exactly this "update in place"
shape.

### Scenario: Querying a database

After a bulk query returns a batch of rows, building a `HashMap` keyed
by ID turns repeated "find the row for this ID" lookups from a linear
scan over the batch into an O(1) average lookup.

```
// [dependencies] sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }, tokio = { version = "1", features = ["full"] }
use std::collections::HashMap;

#[derive(sqlx::FromRow, Clone)]
struct Order {
    id: i64,
    total_cents: i64,
}

async fn orders_by_id(pool: &sqlx::PgPool) -> sqlx::Result<HashMap<i64, Order>> {
    let rows: Vec<Order> = sqlx::query_as::<_, Order>("SELECT id, total_cents FROM orders")
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|order| (order.id, order)).collect()) // <- collects straight into a HashMap keyed by id
}
```

**Why this way:** fetching once and indexing the results locally avoids
a separate round-trip query per lookup, and `HashMap`'s O(1) average
`get` keeps that in-memory lookup cheap regardless of batch size, per
the [std docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
on its performance characteristics.

### Scenario: Implementing traits

A struct used as a `HashMap` key needs `Eq` and `Hash` alongside
`PartialEq`, and deriving all three is correct only when equal instances
are guaranteed to hash the same way.

```
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)] // <- Hash + Eq are exactly what HashMap requires of its key type
struct UserId {
    tenant: u32,
    id: u32,
}

let mut sessions: HashMap<UserId, u64> = HashMap::new();
sessions.insert(UserId { tenant: 1, id: 42 }, 1_690_000_000);
```

**Why this way:** the
[std docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html#examples)
require key types to implement `Eq` and `Hash` consistently — deriving
both together on a struct of already-`Hash`+`Eq` fields is the
straightforward way to satisfy that without hand-writing either impl.

## Embedded Rust Notes

**Partial support.** Unlike `Vec`/`BTreeMap`, `HashMap`/`HashSet` are not
part of the `alloc` crate at all: `alloc::collections` only ships the
ordered, hasher-free trees, because a hash table needs a hasher, and
`std`'s default `RandomState` hasher seeds itself from OS randomness to
resist hash-flooding attacks — something `#![no_std]` has no access to.
`no_std` code typically reaches for the `hashbrown` crate directly (the
same hash-table implementation `std::collections::HashMap` is built on
internally, usable with `alloc` and a fixed, non-random hasher), or for
a fixed-capacity alternative like `heapless::FnvIndexMap` when no
allocator is available either.

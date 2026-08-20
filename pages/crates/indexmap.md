---
title: "indexmap"
version: "2.14.0"
publisher: "bluss, Josh Stone (cuviper)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-20"
summary: "A hash map that remembers insertion order and can be indexed by position. Hash lookup like `HashMap`, deterministic iteration like a `Vec`, and sorting without changing the type."
categories: ["data-structures", "collections", "no-std"]
repository: "https://github.com/indexmap-rs/indexmap"
---

## Overview

[`HashMap`](../concepts/collections-strings/hashmap-and-hashset.md) gives you
O(1) lookup and iteration in an order that is deliberately unspecified — and
which changes between runs, because the hasher is randomly seeded. That is the
right default, right up until the order matters: a config file you re-serialise,
a table you print, a test asserting on output.

The usual workarounds are worse than they look. `BTreeMap` gives you *sorted*
order, not the order things arrived in, and needs `Ord`. Collecting into a `Vec`
and sorting at the end loses the lookups you needed on the way. Keeping a `Vec`
alongside the map means two structures that can disagree.

`IndexMap` is a hash map with the entries in a `Vec` and a hash table of indices
beside it. So it does both:

```
use indexmap::IndexMap;

let mut headers = IndexMap::new();
headers.insert("host", "example.com");
headers.insert("accept", "text/html");
headers.insert("user-agent", "curl/8");

// Hash lookup, as with HashMap.
assert_eq!(headers.get("accept"), Some(&"text/html"));

// Iteration in insertion order, every run.
let names: Vec<_> = headers.keys().copied().collect();
assert_eq!(names, ["host", "accept", "user-agent"]);

// And addressable by position.
assert_eq!(headers.get_index(0), Some((&"host", &"example.com")));
```

**The one thing to get right is removal.** Because entries live in a `Vec`,
taking one out of the middle either shifts everything after it down or swaps the
last entry into the hole. `IndexMap` refuses to choose for you:

- `shift_remove` preserves order and is **O(n)** — it moves every later entry.
- `swap_remove` is **O(1)** but moves the last entry into the vacated slot, so
  the order changes.

Plain `remove` exists and is deprecated precisely because it silently did the
swapping one. If you reach for `remove` out of `HashMap` habit you get a
deprecation warning telling you to decide, which is the crate being careful
rather than awkward.

The costs are modest and worth knowing. Memory is higher than `HashMap` — the
entries plus an index table. Lookups carry one extra indirection, so they are a
little slower, while iteration is *faster*, being a straight walk over a `Vec`.
It is a mature crate with just two dependencies — `equivalent`, and
[`hashbrown`](hashbrown.md), whose raw table it uses for the index side. It
requires Rust 1.85 and works without `std` given `alloc`.

Reach for `HashMap` when order genuinely doesn't matter, `BTreeMap` when you
want sorted order and have `Ord`, and `IndexMap` when you want the order things
happened in.

## When to use it

### Use case: Round-tripping a config file without reordering it

Deserialising a config into a `HashMap` and writing it back shuffles the file.
Every key moves, the diff is unreadable, and the user's careful grouping is
gone.

```
use indexmap::IndexMap;

// As parsed from the file, in file order.
let mut settings: IndexMap<String, String> = IndexMap::new();
settings.insert("name".into(), "server".into());
settings.insert("port".into(), "8080".into());
settings.insert("workers".into(), "4".into());

// Change one value; everything else stays where the user put it.
if let Some(port) = settings.get_mut("port") {
    *port = "9090".into();
}

let rendered: Vec<String> = settings
    .iter()
    .map(|(k, v)| format!("{k} = {v}"))
    .collect();

assert_eq!(rendered, ["name = server", "port = 9090", "workers = 4"]);
```

**Why it fits:** the rewritten file differs from the original on exactly the line
that changed. With a `HashMap` the same edit produces a diff touching every line,
which makes review useless and version control noisy.

### Use case: Deterministic output in tests

A test asserting on a rendered map is flaky with `HashMap` — the order varies per
run because the hasher is seeded randomly. `IndexMap` makes the output a
function of the input.

```
use indexmap::IndexMap;

fn summarise(events: &[(&str, u32)]) -> String {
    let mut totals: IndexMap<&str, u32> = IndexMap::new();
    for (name, count) in events {
        *totals.entry(name).or_insert(0) += count;
    }
    totals
        .iter()
        .map(|(name, total)| format!("{name}={total}"))
        .collect::<Vec<_>>()
        .join(",")
}

// First-seen order, so the expected string is stable.
let out = summarise(&[("b", 1), ("a", 2), ("b", 3)]);
assert_eq!(out, "b=4,a=2");
```

**Why it fits:** the assertion can name the whole output instead of sorting it
first or comparing sets. Note what decides the order — first insertion, so `b`
leads despite `a` being alphabetically first and despite `b` being updated
later.

### Use case: An interner, or anything keyed by position

When you need a small integer for each distinct value, `IndexMap` gives you one
for free: the entry's index is a dense id, and it maps back both ways.

```
use indexmap::IndexSet;

let mut symbols: IndexSet<String> = IndexSet::new();

let (id_a, _) = symbols.insert_full("alpha".to_string());
let (id_b, _) = symbols.insert_full("beta".to_string());
let (id_a_again, inserted) = symbols.insert_full("alpha".to_string());

assert_eq!((id_a, id_b), (0, 1));
assert_eq!(id_a_again, id_a);
assert!(!inserted); // <- already present

// id -> value, and value -> id.
assert_eq!(symbols.get_index(id_b), Some(&"beta".to_string()));
assert_eq!(symbols.get_index_of("alpha"), Some(0));
```

**Why it fits:** the alternative is a `HashMap<String, u32>` plus a
`Vec<String>` kept in step by hand, which is two structures and an invariant to
maintain. Ids stay valid as long as you only ever push and use `shift_remove` —
`swap_remove` renumbers whatever was last.

## API map

`IndexMap` mirrors `HashMap`'s API closely, so the entries below concentrate on
what is genuinely different: the ordering guarantees, indexed access, the two
removals, and sorting. `IndexSet` is the same structure with no values and the
same methods.

### Creating and inserting

#### `IndexMap::new`

An empty map with the default hasher.

```
use indexmap::IndexMap;

let mut map: IndexMap<&str, i32> = IndexMap::new();
map.insert("a", 1);

assert_eq!(map.len(), 1);
assert!(!map.is_empty());
```

**When to use it:** the default constructor. `with_capacity` avoids reallocating
when you know the size, and matters slightly more here than for `HashMap`,
because two structures grow rather than one.

#### `insert`

Inserts, returning the previous value. **An existing key keeps its position.**

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

// Updating `a` does not move it to the end.
assert_eq!(map.insert("a", 10), Some(1));
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["a", "b"]);
```

**When to use it:** the ordinary write. The position rule is the one to
remember — order reflects *first* insertion, not last update, which is usually
what you want and occasionally not. `shift_insert` puts an entry at a chosen
index instead.

#### `insert_full`

Inserts and also reports the entry's index.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
let (index, previous) = map.insert_full("first", 100);

assert_eq!(index, 0);
assert_eq!(previous, None);

let (index, previous) = map.insert_full("first", 200);
assert_eq!(index, 0); // <- same slot
assert_eq!(previous, Some(100));
```

**When to use it:** interning and id assignment, where the index *is* the value
you want. It saves the extra `get_index_of` that the plain `insert` would need
afterwards.

#### `entry`

The same entry API as `HashMap`, with a vacant insert landing at the end.

```
use indexmap::IndexMap;

let mut counts: IndexMap<&str, u32> = IndexMap::new();
for word in ["b", "a", "b"] {
    *counts.entry(word).or_insert(0) += 1;
}

assert_eq!(counts["b"], 2);
assert_eq!(counts.keys().copied().collect::<Vec<_>>(), ["b", "a"]);
```

**When to use it:** accumulating into a map, exactly as with `HashMap`. The
entry also exposes `index()`, so you can learn where the value landed without a
second lookup.

### Lookup by key

#### `get` and `get_mut`

Hash lookup, unchanged from `HashMap`.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("key", 1);

assert_eq!(map.get("key"), Some(&1));
if let Some(v) = map.get_mut("key") {
    *v += 1;
}
assert_eq!(map["key"], 2);
```

**When to use it:** any read where position is irrelevant. These are O(1) and
carry one more indirection than `HashMap`'s — the table stores an index, which
is then used to reach the entry.

#### `get_index_of`

The position of a key, which is the bridge from key-space into index-space.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

assert_eq!(map.get_index_of("b"), Some(1));
assert_eq!(map.get_index_of("missing"), None);
```

**When to use it:** when you need to know where something sits — to compare two
keys' order, to slice around it, or to store a compact id. Remember the index is
only stable while the map isn't `swap_remove`d or sorted.

#### `get_full`

Index, key and value in one lookup.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

assert_eq!(map.get_full("b"), Some((1, &"b", &2)));
```

**When to use it:** when you want more than the value and would otherwise call
`get` and `get_index_of` separately. `get_full_mut` is the mutable form.

### Lookup by position

#### `get_index`

The entry at a position, as `HashMap` cannot do at all.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

assert_eq!(map.get_index(0), Some((&"a", &1)));
assert_eq!(map.get_index(9), None); // <- out of range, not a panic
```

**When to use it:** rendering a numbered list, paging through entries, or
resolving an id you handed out earlier. It returns `Option`, so an id that has
gone stale gives you `None` rather than a panic.

#### `first` and `last`

The ends of the map.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

assert_eq!(map.first(), Some((&"a", &1)));
assert_eq!(map.last(), Some((&"b", &2)));
```

**When to use it:** treating the map as a queue or a history — oldest and newest
entry. `pop` removes and returns the last pair, which with `insert` makes a
stack whose members are also key-addressable.

#### `as_slice`

The entries as a `Slice`, which indexes and iterates like a slice of pairs.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

let middle = map.get_range(1..).unwrap();
assert_eq!(middle.len(), 2);
assert_eq!(map.as_slice().first(), Some((&"a", &1)));
```

**When to use it:** when the map is really an ordered sequence for a moment —
taking a range, binary searching a sorted map, handing a window to something
that wants a slice.

### Removing

#### `shift_remove`

Removes and **keeps the order**, shifting later entries down. O(n).

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

assert_eq!(map.shift_remove("b"), Some(2));
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["a", "c"]);
```

**When to use it:** whenever order is the reason you chose this crate — config
files, rendered output, anything a person reads. The O(n) cost is the price of
that, and is irrelevant for the small maps this usually applies to. Removing
many entries at once is better done with `retain`, which is a single pass.

#### `swap_remove`

Removes in **O(1)** by moving the last entry into the gap, changing the order.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

assert_eq!(map.swap_remove("a"), Some(1));
// "c" was last, and has taken the vacated slot.
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["c", "b"]);
```

**When to use it:** when the map is a set you're draining and the order is
incidental. Never when someone is holding an index — the moved entry silently
changes id, and nothing will tell you.

#### `retain`

Keeps the entries matching a predicate, preserving order, in one pass.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

map.retain(|_, v| *v % 2 == 1);

assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["a", "c"]);
```

**When to use it:** filtering in place. Preferable to a loop of `shift_remove`,
which is O(n) per call and so O(n²) overall; `retain` is O(n) for the lot.

### Sorting and reordering

#### `sort_keys` and `sort_by`

Sorts the entries in place, without changing the type.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("c", 3);
map.insert("a", 1);
map.insert("b", 2);

map.sort_keys();
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["a", "b", "c"]);

// Or by anything: here, value descending.
map.sort_by(|_, a, _, b| b.cmp(a));
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["c", "b", "a"]);
```

**When to use it:** presenting a map that was built in arrival order — sort once
at the end and keep hash lookup throughout, which a `BTreeMap` cannot offer and
a `Vec` gives up. `sort_unstable_by` is faster when equal elements needn't keep
their relative order.

#### `move_index` and `swap_indices`

Reorder entries directly.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

map.move_index(2, 0); // <- "c" to the front, others shift
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["c", "a", "b"]);

map.swap_indices(0, 2); // <- exchange two positions
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["b", "a", "c"]);
```

**When to use it:** implementing user-facing reordering — dragging a row,
promoting an item. `move_index` shifts the entries between, like
`shift_remove`; `swap_indices` touches only the two, like `swap_remove`.

#### `reverse`

Reverses the entry order in place.

```
use indexmap::IndexMap;

let mut map = IndexMap::new();
map.insert("a", 1);
map.insert("b", 2);

map.reverse();
assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["b", "a"]);
```

**When to use it:** flipping newest-first to oldest-first. Cheaper and clearer
than sorting by an index you'd have to fabricate, and it keeps every key
reachable throughout.

---
title: "itertools"
version: "0.15.0"
publisher: "bluss, Jack Wrenn (jswrenn)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-26"
summary: "Extra iterator adaptors, as one extension trait: grouping, chunking, deduplication, combinations, and the `Result`-aware adaptors that let a fallible pipeline stay a pipeline."
categories: ["iterators", "algorithms", "no-std"]
repository: "https://github.com/rust-itertools/itertools"
---

## Overview

Rust's [iterators](../concepts/iterators/iterator-adaptors.md) cover the common
cases well, and then you want to group consecutive equal items, or pair each
element with the next, or turn an iterator of `Result` into a `Result` of a
collection without giving up on the chain. Those all exist; none of them are in
`std`.

`itertools` is one extension trait, `Itertools`, implemented for every
`Iterator`. Import it and roughly two hundred extra methods appear on iterators
you already have:

```
use itertools::Itertools;

let words = ["apple", "avocado", "banana", "blueberry", "cherry"];

// Group consecutive items by first letter, then render each group.
let grouped: Vec<String> = words
    .iter()
    .chunk_by(|w| w.chars().next().unwrap())
    .into_iter()
    .map(|(letter, group)| format!("{letter}: {}", group.count()))
    .collect();

assert_eq!(grouped, ["a: 2", "b: 2", "c: 1"]);
```

Nothing here is impossible without it. The value is that the pipeline stays a
pipeline — the alternative is usually a `for` loop with a mutable accumulator
and an `if` for the boundary case, which is longer and easier to get wrong at
the edges.

**Two things to know before reaching for it.**

The first is that `std` keeps catching up. `zip`, `flatten`, `is_sorted` and
several others were once reasons to depend on itertools and no longer are. Check
the standard library first: a method that exists in both is better taken from
`std`, and the version numbers here move because the crate genuinely churns.

The second is a shape that surprises people. `chunks` and `chunk_by` return a
value that is **not** an iterator — it implements `IntoIterator`, because the
group iterators borrow from it. So it has to be bound to a local, and
`.into_iter()` called on that; chaining straight through gives a borrow error
that reads as though you have done something much stranger than you have. The
Overview example above shows the shape.

It is a mature crate, one of the oldest in the ecosystem, with a single
dependency (`either`). MSRV is 1.63, and it works without `std`: `use_alloc`
covers the adaptors needing allocation, and turning both features off leaves the
ones that need none.

Reach for it when a loop is starting to accumulate state that an adaptor would
name, and leave it out when `std`'s iterators already say what you mean.

## When to use it

### Use case: Collecting a fallible pipeline

An iterator of `Result` is awkward: filtering and mapping have to unwrap first,
and the first error should abort. `process_results` hands the pipeline a plain
iterator of values and returns the first error if one appears.

```
use itertools::Itertools;

fn total(inputs: &[&str]) -> Result<i64, std::num::ParseIntError> {
    inputs
        .iter()
        .map(|s| s.parse::<i64>())
        .process_results(|iter| iter.filter(|n| *n > 0).sum())
}

assert_eq!(total(&["1", "-2", "3"]), Ok(4));
assert!(total(&["1", "oops"]).is_err());
```

**Why it fits:** the closure sees `i64`, not `Result<i64, _>`, so the filter and
the sum read normally. `collect::<Result<Vec<_>, _>>()` would work too, but
allocates the whole vector before summing it; this short-circuits at the first
error without collecting at all.

### Use case: Grouping records by a key

Building a `HashMap<K, Vec<V>>` by hand is four lines of `entry().or_default()`.
`into_group_map` is the same thing named.

```
use itertools::Itertools;

let logs = [("api", 120), ("db", 45), ("api", 80), ("cache", 5)];

let by_service = logs.into_iter().into_group_map();

assert_eq!(by_service["api"], vec![120, 80]);
assert_eq!(by_service["db"], vec![45]);
assert_eq!(by_service.len(), 3);
```

**Why it fits:** the grouping is the point of the code, so it should be the verb
in it. Unlike `chunk_by`, this collects — it groups across the whole iterator
rather than only consecutive runs, which is nearly always what you want when the
input isn't sorted.

### Use case: Comparing each element with the next

Anything that looks at neighbours — gaps in a sequence, deltas between samples,
checking sortedness by hand — wants a sliding window, which `std` offers on
slices but not on iterators.

```
use itertools::Itertools;

let readings = [10, 12, 11, 20];

let deltas: Vec<i32> = readings
    .iter()
    .tuple_windows()
    .map(|(a, b)| b - a)
    .collect();

assert_eq!(deltas, [2, -1, 9]);

// The biggest jump, without indexing anything.
let spike = deltas.iter().copied().max().unwrap();
assert_eq!(spike, 9);
```

**Why it fits:** `windows` exists on slices, so this matters exactly when the
data is an iterator you don't want to collect first — a file's lines, a channel,
a parser's output. The tuple form also gives you named elements rather than a
slice you have to index.

## API map

The trait has around two hundred methods, so this is a curated selection rather
than a listing: the ones that come up repeatedly, grouped by what you are trying
to do. Everything below needs `use itertools::Itertools;` in scope, since these
are trait methods.

### Grouping and chunking

#### `chunk_by`

Groups **consecutive** elements sharing a key.

```
use itertools::Itertools;

let data = [1, 1, 2, 2, 2, 1];

// Bound to a local: the groups borrow from it.
let runs = data.iter().chunk_by(|n| **n);
let summary: Vec<(i32, usize)> = runs
    .into_iter()
    .map(|(key, group)| (key, group.count()))
    .collect();

assert_eq!(summary, [(1, 2), (2, 3), (1, 1)]); // <- 1 appears twice: runs, not groups
```

**When to use it:** run-length encoding, splitting sorted data into sections,
paragraph-by-blank-line parsing. Only consecutive items group, so sort first if
you want all equal keys together — or use `into_group_map`, which doesn't care
about order.

#### `chunks`

Fixed-size batches, in order.

```
use itertools::Itertools;

let ids = 1..=7;

let batches = ids.chunks(3);
let sizes: Vec<usize> = batches.into_iter().map(|c| c.count()).collect();

assert_eq!(sizes, [3, 3, 1]); // <- the last chunk is short
```

**When to use it:** batching work — a hundred rows per insert, a page of results
per request. Like `chunk_by` it must be bound to a local first. For a slice,
`slice::chunks` is simpler and gives you slices back rather than iterators.

#### `into_group_map`

Collects `(key, value)` pairs into a `HashMap<K, Vec<V>>`.

```
use itertools::Itertools;

let pairs = [("a", 1), ("b", 2), ("a", 3)];
let map = pairs.into_iter().into_group_map();

assert_eq!(map["a"], vec![1, 3]);
```

**When to use it:** grouping unsorted data by key in one call.
`into_group_map_by` takes a closure instead, for when the items aren't already
pairs. Both allocate a `Vec` per key, so `into_grouping_map` is better when
you're going to reduce each group anyway.

#### `into_grouping_map`

Groups and folds in one pass, without building the intermediate `Vec`s.

```
use itertools::Itertools;

let sales = [("north", 10), ("south", 5), ("north", 7)];

let totals = sales.into_iter().into_grouping_map().sum();
assert_eq!(totals["north"], 17);

let sales = [("north", 10), ("south", 5), ("north", 7)];
let best = sales.into_iter().into_grouping_map().max();
assert_eq!(best["north"], 10);
```

**When to use it:** aggregating by key — totals, maxima, counts. It carries
`sum`, `max`, `min`, `fold` and `collect`, and skips the per-key `Vec` that
`into_group_map` would allocate and then immediately consume.

### Combining iterators

#### `zip_eq`

Zips two iterators, panicking if their lengths differ.

```
use itertools::Itertools;

let names = ["a", "b", "c"];
let scores = [1, 2, 3];

let paired: Vec<(&str, i32)> = names.iter().copied().zip_eq(scores).collect();
assert_eq!(paired.len(), 3);
```

**When to use it:** when the two sequences are supposed to correspond and a
length mismatch is a bug. `std`'s `zip` silently stops at the shorter one, which
turns a lost row into a shorter output nobody notices. Don't use it on input you
don't control — the panic is a poor error.

#### `zip_longest`

Zips to the length of the *longer*, yielding `EitherOrBoth`.

```
use itertools::{EitherOrBoth, Itertools};

let old = [1, 2];
let new = [1, 2, 3];

let changes: Vec<String> = old
    .iter()
    .zip_longest(new.iter())
    .map(|pair| match pair {
        EitherOrBoth::Both(a, b) if a == b => "same".to_string(),
        EitherOrBoth::Both(a, b) => format!("{a} -> {b}"),
        EitherOrBoth::Left(a) => format!("removed {a}"),
        EitherOrBoth::Right(b) => format!("added {b}"),
    })
    .collect();

assert_eq!(changes, ["same", "same", "added 3"]);
```

**When to use it:** diffing two sequences, or merging where either side may run
out. The `EitherOrBoth` match makes every case explicit, which is the difference
between a diff that reports additions and one that quietly drops them.

#### `interleave` and `merge`

Alternate between two iterators, or merge two sorted ones in order.

```
use itertools::Itertools;

let a = [1, 3, 5];
let b = [2, 4, 6];

let alternating: Vec<i32> = a.iter().copied().interleave(b.iter().copied()).collect();
assert_eq!(alternating, [1, 2, 3, 4, 5, 6]);

let merged: Vec<i32> = [1, 4, 7].into_iter().merge([2, 3, 9]).collect();
assert_eq!(merged, [1, 2, 3, 4, 7, 9]); // <- sorted inputs stay sorted
```

**When to use it:** `interleave` for round-robin scheduling and alternating
sources; `merge` for combining sorted streams without collecting and re-sorting.
`merge` assumes both inputs are already sorted and produces nonsense if they
aren't — it cannot check.

#### `cartesian_product`

Every pair from two iterators.

```
use itertools::Itertools;

let coords: Vec<(i32, char)> = (1..=2).cartesian_product(['a', 'b']).collect();

assert_eq!(coords, [(1, 'a'), (1, 'b'), (2, 'a'), (2, 'b')]);
```

**When to use it:** grids, test matrices, every combination of two option sets.
It is a nested loop written as an expression, so it composes with `filter` and
`map`. `iproduct!` takes more than two.

### Windows and tuples

#### `tuple_windows`

Overlapping windows, as tuples.

```
use itertools::Itertools;

let values = [1, 2, 3, 4];

let triples: Vec<(i32, i32, i32)> = values.iter().copied().tuple_windows().collect();
assert_eq!(triples, [(1, 2, 3), (2, 3, 4)]);
```

**When to use it:** comparing neighbours in an iterator. The window size comes
from the tuple's arity, so annotate the type. It clones elements, which is free
for `Copy` and worth noting for anything larger.

#### `tuples`

Non-overlapping groups, as tuples.

```
use itertools::Itertools;

let flat = [1, 2, 3, 4, 5];

let pairs: Vec<(i32, i32)> = flat.iter().copied().tuples().collect();
assert_eq!(pairs, [(1, 2), (3, 4)]); // <- the leftover 5 is dropped
```

**When to use it:** reading a flat sequence that comes in fixed-size records —
coordinate pairs, key/value runs. Note the truncation: a trailing partial group
is discarded silently, so check the length first if that would be data loss.

#### `collect_tuple`

Collects into a tuple, or `None` if the count doesn't match exactly.

```
use itertools::Itertools;

let parts: Option<(&str, &str)> = "key=value".split('=').collect_tuple();
assert_eq!(parts, Some(("key", "value")));

let wrong: Option<(&str, &str)> = "a=b=c".split('=').collect_tuple();
assert_eq!(wrong, None); // <- three parts, not two
```

**When to use it:** parsing something with an exact expected shape. It is the
concise, total alternative to `next().unwrap()` twice — the `None` tells you the
input was malformed rather than panicking or silently ignoring the extra.

### Deduplicating

#### `unique`

Yields each distinct element once, keeping first-seen order.

```
use itertools::Itertools;

let values = [3, 1, 3, 2, 1];
let distinct: Vec<i32> = values.iter().copied().unique().collect();

assert_eq!(distinct, [3, 1, 2]);
```

**When to use it:** deduplicating unsorted data lazily. It holds a `HashSet` of
what it has seen, so it needs `Hash + Eq` and memory proportional to the number
of distinct items. `unique_by` takes a key function for when only part of the
element identifies it.

#### `dedup`

Collapses **consecutive** duplicates.

```
use itertools::Itertools;

let values = [1, 1, 2, 1];
let collapsed: Vec<i32> = values.iter().copied().dedup().collect();

assert_eq!(collapsed, [1, 2, 1]); // <- the trailing 1 survives
```

**When to use it:** cleaning up runs in sorted or naturally-grouped data. It
keeps no state beyond the last element, so it is O(1) in memory where `unique`
is O(n) — the trade being that it only sees neighbours.
`dedup_with_count` reports how many were collapsed.

#### `all_unique`

Whether every element is distinct.

```
use itertools::Itertools;

assert!([1, 2, 3].iter().all_unique());
assert!(![1, 2, 2].iter().all_unique());
```

**When to use it:** validating that identifiers don't repeat — config keys,
column names. It short-circuits on the first repeat, so it is cheaper than
collecting into a set and comparing lengths.

### Sorting and selecting

#### `sorted_by_key`

Sorts and returns an iterator, so a pipeline doesn't have to break for a
`Vec`.

```
use itertools::Itertools;

let words = ["ccc", "a", "bb"];

let shortest_two: Vec<&str> = words.iter().copied().sorted_by_key(|w| w.len()).take(2).collect();
assert_eq!(shortest_two, ["a", "bb"]);
```

**When to use it:** when sorting is a step in the middle rather than the end. It
collects internally — sorting cannot be lazy — so it is a convenience rather
than a saving, and `k_smallest` is the better answer when you only want a few.

#### `k_smallest`

The k smallest elements, without sorting the rest.

```
use itertools::Itertools;

let values = [9, 1, 8, 2, 7];
let bottom: Vec<i32> = values.iter().copied().k_smallest(2).collect();

assert_eq!(bottom, [1, 2]);
```

**When to use it:** top-N over a large input. It keeps a heap of size k rather
than sorting everything, so the cost is O(n log k) and the memory is k — the
difference that matters when n is a log file and k is ten. `k_largest` is the
other direction.

#### `minmax`

Both extremes in a single pass.

```
use itertools::{Itertools, MinMaxResult};

let values = [3, 1, 4, 1, 5];

match values.iter().copied().minmax() {
    MinMaxResult::MinMax(lo, hi) => assert_eq!((lo, hi), (1, 5)),
    other => panic!("unexpected: {other:?}"),
}

assert_eq!(std::iter::empty::<i32>().minmax(), MinMaxResult::NoElements);
```

**When to use it:** when you need both bounds and the iterator is expensive or
single-use. `MinMaxResult` distinguishes empty from one-element, which
`min()`/`max()` returning two `Option`s cannot do as clearly.

#### `position_max`

The index of the largest element.

```
use itertools::Itertools;

let scores = [10, 40, 25];
assert_eq!(scores.iter().position_max(), Some(1));
assert_eq!(std::iter::empty::<i32>().position_max(), None);
```

**When to use it:** when you need *where* the maximum is, not just what it is —
picking the winning column, locating a peak. Doing it with `enumerate` and
`max_by_key` works but reads as a puzzle.

### Results and output

#### `map_ok` and `filter_ok`

Transform the `Ok` side of an iterator of `Result`, leaving errors alone.

```
use itertools::Itertools;

let parsed: Vec<Result<i32, String>> = vec![Ok(1), Err("bad".to_string()), Ok(3)];

let doubled: Vec<Result<i32, String>> = parsed
    .into_iter()
    .map_ok(|n| n * 2)
    .filter_ok(|n| *n > 2)
    .collect();

assert_eq!(doubled, [Err("bad".to_string()), Ok(6)]);
```

**When to use it:** when errors must survive to the end rather than
short-circuit — collecting every failure to report at once. Use
`process_results` instead when the first error should abort.

#### `partition_result`

Splits an iterator of `Result` into successes and failures.

```
use itertools::Itertools;

let inputs = ["1", "x", "3"];
let (ok, failed): (Vec<i32>, Vec<_>) = inputs
    .iter()
    .map(|s| s.parse::<i32>())
    .partition_result();

assert_eq!(ok, [1, 3]);
assert_eq!(failed.len(), 1);
```

**When to use it:** validating a batch where partial success is meaningful —
import 900 rows and report the 100 that failed. `collect::<Result<Vec<_>, _>>()`
throws away both the successes and every error after the first.

#### `join` and `format`

Render an iterator as a string.

```
use itertools::Itertools;

let names = ["a", "b", "c"];

assert_eq!(names.iter().join(", "), "a, b, c");

// format borrows and writes lazily — no intermediate String.
let rendered = format!("[{}]", names.iter().format(" | "));
assert_eq!(rendered, "[a | b | c]");
```

**When to use it:** `join` where `std`'s only exists on slices and requires
`Vec<String>` first; `format` inside a larger `format!`, where it writes straight
into the formatter and allocates nothing. `format_with` takes a closure when
each element needs more than `Display`.

#### `collect_vec`

`collect::<Vec<_>>()`, without the turbofish.

```
use itertools::Itertools;

let squares = (1..=3).map(|n| n * n).collect_vec();
assert_eq!(squares, [1, 4, 9]);
```

**When to use it:** at the end of a chain where the element type is obvious and
the turbofish is noise. It is pure convenience with no behavioural difference —
worth knowing mainly because you will read it in other people's code.

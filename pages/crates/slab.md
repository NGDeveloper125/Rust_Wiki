---
title: "slab"
version: "0.4.12"
publisher: "Carl Lerche (carllerche), Core"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-08"
summary: "Pre-allocated storage that hands you a `usize` key for each value you insert. Constant-time insert, lookup and remove with no hashing and no per-element allocation — the structure behind connection tables and arenas."
domain: "Collections & data structures"
categories: ["data-structures", "memory-management", "no-std"]
repository: "https://github.com/tokio-rs/slab"
docs: "https://docs.rs/slab/0.4.12/slab/"
---

## Overview

A `Slab<T>` is a `Vec<T>` with a free list. You hand it a value, it gives you
back a `usize` key; you hand the key back to read, mutate or remove. Removing
leaves a hole that the next insert fills.

```
use slab::Slab;

let mut slab = Slab::new();

// The slab chooses the key, not you.
let hello = slab.insert("hello");
let world = slab.insert("world");

assert_eq!(slab[hello], "hello");
assert_eq!(slab.remove(world), "world");

// The freed slot is handed out again.
assert_eq!(slab.insert("again"), world);
```

That inversion is the whole idea, and it is what distinguishes a slab from a
`HashMap<usize, T>`. **You do not choose the key — the slab does.** In exchange,
insert, lookup and remove are all constant time with no hashing, values live in
one contiguous allocation, and there is no per-element `Box`. Lookup is an index
and a tag check.

It solves a specific problem: you have many objects of one type, they come and
go, and you need a cheap handle to each. That is a connection table, a timer
wheel, a set of in-flight requests, the nodes of a graph. It lives in the Tokio
organisation, and `tokio-util` builds its `DelayQueue` on it — timer entries come
and go, and the key is the handle you cancel with.

**The thing to understand before using it: keys are recycled.** Nothing detects
a stale key. After `remove(world)`, the key `world` is not invalid — it is
*vacant*, and the very next insert makes it valid again, pointing at an entirely
different value. `get` will cheerfully return that value, and `contains` will
say `true`.

```
use slab::Slab;

let mut slab = Slab::new();
let key = slab.insert("original");
slab.remove(key);

// Reusing the slot silently revives the old key.
slab.insert("different value");
assert_eq!(slab.get(key), Some(&"different value")); // <- not None
```

If stale handles can reach your slab — stored in a message, sent to another
task, written into a client response — that silent revival is a bug waiting to
happen, and no amount of care at the call site prevents it. The fix is a
*generational* arena, where each key carries a counter that invalidates on
removal: `slotmap` and `generational-arena` both do this, at the cost of a
larger key and a comparison per lookup. Reach for a slab when keys are internal
and their lifetime is something you control; reach for a generational arena when
they are not.

Two smaller caveats. **Capacity does not shrink on remove** — a slab that peaked
at ten thousand connections keeps room for ten thousand until you call
`shrink_to_fit` or `compact`. And **keys are not stable across `compact`**,
which is why that method takes a closure telling you about every move.

It is a safe dependency to take: no required dependencies, `std` behind a
default feature so `no_std` with `alloc` works by turning it off, MSRV 1.51,
optional `serde` support, and an API that has been stable at 0.4 for years.

## When to use it

### Use case: A table of live connections or sessions

The case the crate was written for.

```
use slab::Slab;

struct Connection {
    peer: String,
    bytes_sent: u64,
}

let mut connections = Slab::new();

// The returned key *is* the connection handle.
let alice = connections.insert(Connection { peer: "alice".into(), bytes_sent: 0 });
let bob = connections.insert(Connection { peer: "bob".into(), bytes_sent: 0 });

connections[alice].bytes_sent += 512;

// A disconnect is one O(1) removal; the slot is reused by the next accept.
let gone = connections.remove(bob);
assert_eq!(gone.peer, "bob");

assert_eq!(connections.len(), 1);
assert_eq!(connections[alice].bytes_sent, 512);
```

**Why it fits:** the alternative is a `HashMap<u64, Connection>` with a counter
you increment yourself, which costs a hash per packet and an allocation per
entry, and still leaves you inventing the ids. The slab gives you a dense,
cache-friendly table and an id that is already an array index. It is also how an
event loop maps a readiness notification back to the socket that caused it:
[`mio`](mio.md)'s `Token` is a newtype around `usize`, so a slab key goes
straight into one.

### Use case: A graph or tree without `Rc` or raw pointers

Arena allocation, where edges are keys instead of references.

```
use slab::Slab;

struct Node {
    value: i32,
    children: Vec<usize>,
}

let mut tree = Slab::new();

let leaf_a = tree.insert(Node { value: 1, children: vec![] });
let leaf_b = tree.insert(Node { value: 2, children: vec![] });
let root = tree.insert(Node { value: 0, children: vec![leaf_a, leaf_b] });

// Walking the tree is index arithmetic, with no lifetime to thread through.
let total: i32 = tree[root].value
    + tree[root]
        .children
        .iter()
        .map(|&k| tree[k].value)
        .sum::<i32>();

assert_eq!(total, 3);
```

**Why it fits:** a tree of `Box<Node>` cannot have a parent pointer without
`Rc<RefCell<_>>`, and a graph with cycles cannot be expressed with ownership at
all. Keys sidestep the borrow checker entirely — a `usize` borrows nothing, so
cycles, back-edges and shared children are all just numbers. The whole structure
also drops in one go, which matters for a large graph where recursive `Box` drop
can overflow the stack.

### Use case: Knowing a value's key before you have the value

`vacant_entry` reserves the slot first, which you need when the value has to
contain its own key.

```
use slab::Slab;

struct Task {
    id: usize,
    label: String,
}

let mut tasks = Slab::new();

let entry = tasks.vacant_entry();
let id = entry.key();
// The value can now record the key it is about to live at.
entry.insert(Task { id, label: format!("task-{id}") });

assert_eq!(tasks[id].label, "task-0");
assert_eq!(tasks[id].id, id);
```

**Why it fits:** without this you would insert a placeholder and patch it
afterwards, which means the field is briefly wrong and every reader has to
tolerate that. `vacant_entry` makes the key available before the value exists,
so the value is correct from the moment it is constructed. The same shape covers
registering a callback that needs to be able to deregister itself.

## API map

The vocabulary is `Vec`'s and `HashMap`'s, with keys the slab issues rather than
ones you supply. Entries are grouped by what they do to the slab.

### Inserting and removing

#### `insert`

Stores a value and returns its key.

```
use slab::Slab;

let mut slab = Slab::new();

// Keys start at 0 and go up while the slab is only growing.
assert_eq!(slab.insert('a'), 0);
assert_eq!(slab.insert('b'), 1);
assert_eq!(slab.len(), 2);
```

**When to use it:** whenever you add a value. It is amortised O(1): it fills the
most recently freed slot if there is one, otherwise it pushes, reallocating the
backing `Vec` on the same schedule a `Vec` would. The key is meaningful only to
this slab — do not persist it, derive anything from its numeric value, or assume
it stays in insertion order once removals start.

#### `vacant_entry` and `vacant_key`

Gets the next key before committing a value to it.

```
use slab::Slab;

let mut slab: Slab<String> = Slab::new();

// A peek, with no reservation: the slab is unchanged.
assert_eq!(slab.vacant_key(), 0);
assert!(slab.is_empty());

// A reservation you can take the key from and then fill.
let entry = slab.vacant_entry();
let key = entry.key();
let value: &mut String = entry.insert(format!("slot {key}"));
value.push('!');

assert_eq!(slab[0], "slot 0!");
```

**When to use it:** `vacant_entry` when the value must know its own key, or when
building the value can fail and you want to decide after seeing the key —
dropping the entry without inserting leaves the slab untouched. `vacant_key` is
the cheap read-only version for when you only need to predict the key.
`VacantEntry::insert` returns `&mut T`, so you can finish initialising in place.

#### `remove` and `try_remove`

Takes a value out and frees its slot.

```
use slab::Slab;

let mut slab = Slab::new();
let key = slab.insert("value");

assert_eq!(slab.try_remove(key), Some("value"));

// Already gone, so the second attempt reports rather than panics.
assert_eq!(slab.try_remove(key), None);
assert!(!slab.contains(key));
```

**When to use it:** `try_remove` by default — it handles a double-remove without
panicking, which is the common shape when a disconnect and a timeout can both
fire for the same connection. `remove` panics on a vacant key and is right only
where the key provably still exists. Neither shrinks the backing storage.

#### `retain`, `drain` and `clear`

Bulk removal.

```
use slab::Slab;

let mut slab: Slab<u32> = (0..6).map(|n| (n as usize, n * 10)).collect();

// Keep the entries whose value and key both interest you.
slab.retain(|key, value| key % 2 == 0 && *value > 0);
assert_eq!(slab.len(), 2); // keys 2 and 4

// drain yields the values and empties the slab, keeping its capacity.
let taken: Vec<u32> = slab.drain().collect();
assert_eq!(taken, vec![20, 40]);
assert!(slab.is_empty());
```

**When to use it:** `retain` for a sweep — expiring idle sessions, dropping
closed sockets — since its closure sees the key as well as the value. `drain`
when you want the values back as you empty it; `clear` when you do not. Both
keep the allocation, which is usually what you want for a table that will fill
up again.

### Reading

#### `get`, `get_mut` and indexing

The two ways to look a key up.

```
use slab::Slab;

let mut slab = Slab::new();
let key = slab.insert(10u32);

// Indexing is concise and panics on a vacant key.
slab[key] += 5;
assert_eq!(slab[key], 15);

// get is the checked form.
assert_eq!(slab.get(key), Some(&15));
assert_eq!(slab.get(999), None);

if let Some(value) = slab.get_mut(key) {
    *value = 0;
}
assert_eq!(slab[key], 0);
```

**When to use it:** index when the key came from this slab and has not been
removed since — inside the loop that owns the table, that is most of the time,
and it reads far better than `.get(k).unwrap()`. Use `get` at any boundary where
the key arrived from elsewhere. Remember that neither distinguishes a recycled
key from the original: `get` returning `Some` means the slot is occupied, not
that it holds what you put there.

#### `contains`

Asks whether a key is occupied.

```
use slab::Slab;

let mut slab = Slab::new();
let key = slab.insert("x");

assert!(slab.contains(key));
slab.remove(key);
assert!(!slab.contains(key));
```

**When to use it:** validating a key before indexing, and little else — if you
are about to read the value anyway, `get` tells you both things at once. Note
what it actually means: this slot is occupied *now*, by whatever currently lives
there.

#### `get_disjoint_mut`

Mutable references to several entries at once.

```
use slab::{GetDisjointMutError, Slab};

let mut slab = Slab::new();
let a = slab.insert(1u32);
let b = slab.insert(2u32);

// Two &mut into one slab, which the borrow checker cannot approve alone.
let [first, second] = slab.get_disjoint_mut([a, b]).unwrap();
std::mem::swap(first, second);
assert_eq!((slab[a], slab[b]), (2, 1));

// The same key twice is rejected rather than aliased.
assert_eq!(
    slab.get_disjoint_mut([a, a]),
    Err(GetDisjointMutError::OverlappingIndices)
);
```

**When to use it:** moving a value between two entries, or updating a node and
its parent together. It is the slab equivalent of `split_at_mut`, and it returns
an error — `OverlappingIndices`, `IndexVacant` or `IndexOutOfBounds` — rather
than letting you alias. `get2_mut` is the older two-key form, returning
`Option<(&mut T, &mut T)>`.

#### `key_of`

Recovers the key from a reference into the slab.

```
use slab::Slab;

let mut slab = Slab::new();
let key = slab.insert(String::from("value"));

let value = &slab[key];
assert_eq!(slab.key_of(value), key);
```

**When to use it:** inside a loop that holds a reference and needs the key to
record elsewhere. It is constant time — the key is pointer arithmetic against
the base of the backing `Vec`. It **panics** if the reference does not point
into this slab, and it cannot tell a foreign reference from a valid one by
value, so only pass references you obtained from this slab.

### Iterating and sizing

#### `iter` and `iter_mut`

Walks the occupied entries, keys included.

```
use slab::Slab;

let mut slab = Slab::new();
slab.insert(1u32);
let middle = slab.insert(2);
slab.insert(3);
slab.remove(middle);

// Vacant slots are skipped; keys ascend but are not contiguous.
let seen: Vec<(usize, u32)> = slab.iter().map(|(k, &v)| (k, v)).collect();
assert_eq!(seen, vec![(0, 1), (2, 3)]);

for (_key, value) in slab.iter_mut() {
    *value *= 10;
}
assert_eq!(slab[0], 10);
```

**When to use it:** any sweep over the table — flushing buffers, collecting
stats, expiring entries. The item is `(usize, &T)`, so you get the key without
tracking it yourself. Iteration is O(capacity), not O(len), because it walks the
backing `Vec` and skips holes — a mostly-empty slab with large capacity iterates
slowly until you `compact` it.

#### `with_capacity` and `reserve`

Allocating up front.

```
use slab::Slab;

// One allocation, then 64 inserts that cannot reallocate.
let mut slab: Slab<u64> = Slab::with_capacity(64);
assert!(slab.capacity() >= 64);
assert_eq!(slab.len(), 0);

for n in 0..64 {
    slab.insert(n);
}
assert_eq!(slab.len(), 64);
```

**When to use it:** when you know roughly how many entries you will hold — a
connection limit, a fixed worker count. It is the main reason to prefer a slab
over a map in a hot path: pre-allocate once and inserts stop touching the
allocator. `reserve` and `reserve_exact` grow an existing slab the way their
`Vec` namesakes do.

#### `compact`

Shrinks the slab by moving entries down, telling you each new key.

```
use slab::Slab;

let mut slab = Slab::with_capacity(10);
let a = slab.insert('a');
slab.insert('b');
slab.insert('c');
slab.remove(a);

// 'c' moves from key 2 into the hole at key 0.
slab.compact(|&mut value, from, to| {
    assert_eq!((value, from, to), ('c', 2, 0));
    true // returning false cancels this move and stops
});

assert_eq!(slab.len(), 2);
assert!(slab.capacity() >= 2 && slab.capacity() < 10);
```

**When to use it:** after a burst has drained, when a sparse slab is both wasting
memory and slowing iteration. The closure is how you fix up every key you have
stored elsewhere, and returning `false` aborts a move you cannot rekey — so a
slab whose keys have escaped can still be compacted partially. If you only want
the memory back and have no holes to close, `shrink_to_fit` leaves keys alone.

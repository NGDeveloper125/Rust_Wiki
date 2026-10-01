---
title: "parking_lot"
version: "0.12.5"
publisher: "Linus Färnstrand (faern), Amanieu d'Antras (Amanieu)"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-01"
summary: "Replacements for `std`'s `Mutex`, `RwLock`, `Condvar` and `Once` — smaller, with no lock poisoning, and with timeouts, fairness and reentrancy that `std` doesn't offer."
domain: "Async runtimes & concurrency"
categories: ["concurrency", "synchronization"]
repository: "https://github.com/Amanieu/parking_lot"
---

## Overview

`parking_lot` provides the same synchronisation primitives as the standard
library — `Mutex`, `RwLock`, `Condvar`, `Once` — with different trade-offs. The
most visible difference is the one you hit on the first line:

```
use parking_lot::Mutex;

let counter = Mutex::new(0u32);

// No Result: locking returns the guard directly.
*counter.lock() += 1;
*counter.lock() += 1;

assert_eq!(*counter.lock(), 2);
```

`std::sync::Mutex::lock` returns `Result` because a thread that panics while
holding the lock *poisons* it, and every later lock reports that. The idea is
that the data might be in a half-updated state. In practice almost every caller
writes `.lock().unwrap()`, which converts the poisoning into a panic and throws
the information away. `parking_lot` drops the concept, so the guard comes back
directly.

**The other differences are smaller than the crate's reputation suggests, and
that reputation is dated.** When `parking_lot` appeared, `std`'s mutex was a
heavyweight OS object. Since then `std` moved to futex-based implementations on
Linux, Windows and macOS, and the performance gap narrowed to the point where it
rarely decides anything. What is still true:

- **Size.** `parking_lot::Mutex<()>` is one byte; `std`'s is larger and its
  exact size varies by platform. That matters when you have a lock per element
  across millions of elements, and not otherwise.
- **`const fn new`.** Both have it now, so a `static` works either way.
- **Features `std` lacks.** Timeouts (`try_lock_for`), fair unlocking,
  reentrant mutexes, upgradable read locks, and optional deadlock detection.

So the honest guidance is: **`std` is a fine default**, and `parking_lot` earns
its place when you want no poisoning, need one of those extra features, or have
measured a difference. Reaching for it reflexively is a habit from 2018.

**One thing it is not: async-aware.** These are blocking locks. Holding one
across an `.await` blocks the executor thread, which is the mistake the
[`tokio`](tokio.md) page warns about — and since the guards are not `Send`, a
task doing it usually fails to compile rather than misbehaving quietly. Use
`tokio::sync::Mutex` when a guard must survive an await; use this one (or
`std`'s) when it must not, which is most of the time.

It requires Rust 1.71 and is not `no_std`. It sits in a great many dependency
trees without being asked for directly, which is why its download count runs
well ahead of the number of crates that actually name it.

## When to use it

### Use case: Shared state without `.unwrap()` on every lock

The everyday case, and the clearest difference from `std`.

```
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default)]
struct Stats {
    hits: u64,
    misses: u64,
}

let stats = Arc::new(Mutex::new(Stats::default()));

let mut handles = Vec::new();
for i in 0..4 {
    let stats = Arc::clone(&stats);
    handles.push(std::thread::spawn(move || {
        let mut guard = stats.lock(); // <- no Result to unwrap
        if i % 2 == 0 {
            guard.hits += 1;
        } else {
            guard.misses += 1;
        }
    }));
}
for h in handles {
    h.join().unwrap();
}

let final_stats = stats.lock();
assert_eq!(final_stats.hits, 2);
assert_eq!(final_stats.misses, 2);
```

**Why it fits:** the lock site reads as what it is. With `std` each of those
lines carries an `.unwrap()` that handles a case nobody has a plan for — and the
one place poisoning genuinely helps, recovering deliberately from a panicking
writer, is rare enough that it is worth opting into rather than paying for
everywhere.

### Use case: A lock you refuse to wait forever for

`std` offers `lock` and `try_lock` and nothing between. A timeout is often what
you actually want.

```
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

let resource = Arc::new(Mutex::new("data"));

let held = Arc::clone(&resource);
let holder = std::thread::spawn(move || {
    let _guard = held.lock();
    std::thread::sleep(Duration::from_millis(100));
});

std::thread::sleep(Duration::from_millis(10)); // let the holder acquire it

// Give up rather than blocking a request thread indefinitely.
assert!(resource.try_lock_for(Duration::from_millis(20)).is_none());

holder.join().unwrap();
assert!(resource.try_lock().is_some()); // <- free again
```

**Why it fits:** a request handler that blocks forever on a contended lock is a
hang with no diagnostic. A bounded wait turns it into an error you can report,
retry or shed load on — and `std` has no equivalent, so this is a genuine reason
to take the dependency.

### Use case: Upgrading a read lock to a write lock

Checking under a shared lock and only occasionally needing to write, without
dropping the lock in between.

```
use parking_lot::RwLock;
use std::collections::HashMap;

let cache: RwLock<HashMap<String, u32>> = RwLock::new(HashMap::new());

fn get_or_insert(cache: &RwLock<HashMap<String, u32>>, key: &str) -> u32 {
    // Many readers can hold this at once.
    let upgradable = cache.upgradable_read();
    if let Some(value) = upgradable.get(key) {
        return *value;
    }
    // Only now take exclusive access — without ever releasing the lock.
    let mut write = parking_lot::RwLockUpgradableReadGuard::upgrade(upgradable);
    let value = key.len() as u32;
    write.insert(key.to_string(), value);
    value
}

assert_eq!(get_or_insert(&cache, "alpha"), 5);
assert_eq!(get_or_insert(&cache, "alpha"), 5); // <- served from the map
assert_eq!(cache.read().len(), 1);
```

**Why it fits:** the alternative is to take a read lock, drop it, take a write
lock, and re-check — because another thread may have inserted in the gap. An
upgradable read closes the gap, so the check and the insert are one atomic
decision. Only one upgradable reader is allowed at a time, which is what makes
that safe.

## API map

The types mirror `std::sync`, so the entries below concentrate on where they
differ: no poisoning, extra waiting strategies, and the guards' extra
capabilities.

### `Mutex`

#### `lock`

Takes the lock, returning the guard directly.

```
use parking_lot::Mutex;

let value = Mutex::new(vec![1, 2, 3]);

{
    let mut guard = value.lock();
    guard.push(4);
} // <- released here, as with std

assert_eq!(*value.lock(), vec![1, 2, 3, 4]);
```

**When to use it:** everywhere you would use `std`'s. The guard releases on
drop, so the scope controls the critical section — and because there is no
`Result`, a long chain like `map.lock().entry(k).or_default()` reads without
interruption.

#### `try_lock` and `try_lock_for`

Acquiring without blocking, or with a deadline.

```
use parking_lot::Mutex;
use std::time::Duration;

let lock = Mutex::new(0u8);

// Free, so this succeeds.
assert!(lock.try_lock().is_some());

// Held, so the second attempt gives up immediately.
let _guard = lock.lock();
assert!(lock.try_lock().is_none());
assert!(lock.try_lock_for(Duration::from_millis(5)).is_none());
```

**When to use it:** `try_lock` when there is useful work to do instead of
waiting — skip this item, serve from a stale cache. `try_lock_for` when waiting
a little is fine but waiting forever is not, which is the common shape in a
server. `try_lock_until` takes an `Instant` for a deadline shared across several
operations.

#### `const` construction

Building a lock in a `static`, with no lazy initialisation.

```
use parking_lot::Mutex;

static COUNTER: Mutex<u64> = Mutex::new(0);

*COUNTER.lock() += 5;
assert_eq!(*COUNTER.lock(), 5);
```

**When to use it:** process-wide state — a registry, a counter, a cached
handle. `std::sync::Mutex::new` is also `const` now, so this is no longer a
reason to choose `parking_lot`; it is simply what you would expect to work.

#### `MutexGuard::map`

Narrows a guard to part of the data it protects.

```
use parking_lot::{Mutex, MutexGuard};

struct Config {
    name: String,
    retries: u8,
}

let config = Mutex::new(Config { name: "svc".into(), retries: 3 });

// A guard over just one field, still holding the lock.
let mut retries = MutexGuard::map(config.lock(), |c| &mut c.retries);
*retries = 5;
drop(retries);

assert_eq!(config.lock().retries, 5);
assert_eq!(config.lock().name, "svc");
```

**When to use it:** handing a caller access to one field without exposing the
whole structure, or returning a guard from a function that should not leak the
layout behind it. `std` has no equivalent, so this is another genuine
difference.

### `RwLock`

#### `read` and `write`

Many readers, or one writer.

```
use parking_lot::RwLock;

let data = RwLock::new(vec![1, 2, 3]);

// Several readers may hold this at once.
{
    let a = data.read();
    let b = data.read();
    assert_eq!(a.len(), b.len());
}

data.write().push(4);
assert_eq!(data.read().len(), 4);
```

**When to use it:** data read far more often than written — a routing table, a
configuration, a cache. When reads and writes are balanced, a `Mutex` is usually
faster, because an `RwLock` does more bookkeeping and gains nothing without read
concurrency to exploit.

#### `upgradable_read`

A read lock that can become a write lock without being released.

```
use parking_lot::{RwLock, RwLockUpgradableReadGuard};

let value = RwLock::new(10u32);

let guard = value.upgradable_read();
assert_eq!(*guard, 10);

// Decide to write, atomically with respect to the read.
let mut write = RwLockUpgradableReadGuard::upgrade(guard);
*write += 1;
drop(write);

assert_eq!(*value.read(), 11);
```

**When to use it:** check-then-maybe-write, where the check is the common path.
Only one upgradable reader may exist at a time, so it does not replace `read` for
ordinary readers — take a plain `read` unless you might upgrade.

#### `RwLockWriteGuard::downgrade`

Turning a write lock into a read lock without releasing it.

```
use parking_lot::{RwLock, RwLockWriteGuard};

let data = RwLock::new(String::new());

let mut write = data.write();
write.push_str("prepared");

// Keep reading the value you just wrote, while letting other readers in.
let read = RwLockWriteGuard::downgrade(write);
assert_eq!(&*read, "prepared");
```

**When to use it:** after a write, when you still need the value but no longer
need exclusivity. Dropping and re-reading would let another writer in between,
so this is what you want when the value you just wrote must be the one you then
use.

### Other primitives

#### `Condvar`

Waiting for a condition, with timeouts that report *why* they returned.

```
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::time::Duration;

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let signal = Arc::clone(&pair);

std::thread::spawn(move || {
    std::thread::sleep(Duration::from_millis(10));
    let (lock, cvar) = &*signal;
    *lock.lock() = true;
    cvar.notify_one();
});

let (lock, cvar) = &*pair;
let mut ready = lock.lock();
while !*ready {
    // The guard goes in and comes back out, as with std.
    cvar.wait(&mut ready);
}
assert!(*ready);
```

**When to use it:** a worker waiting for work, a shutdown signal, any
wait-for-a-predicate. The `while` loop is not optional — spurious wakeups are
permitted, so re-checking the condition is required. `wait_for` returns a
`WaitTimeoutResult` that distinguishes a timeout from a notification, which
`std` makes you infer.

#### `Once`

Runs an initialiser exactly once.

```
use parking_lot::Once;
use std::sync::atomic::{AtomicU32, Ordering};

static INIT: Once = Once::new();
static VALUE: AtomicU32 = AtomicU32::new(0);

let mut ran = 0;
for _ in 0..3 {
    INIT.call_once(|| {
        ran += 1;
        VALUE.store(42, Ordering::Relaxed);
    });
}

assert_eq!(ran, 1); // <- the closure ran on the first pass only
assert_eq!(VALUE.load(Ordering::Relaxed), 42);
```

**When to use it:** one-time setup — installing a logger, initialising an FFI
library. For producing a *value* rather than performing an effect,
[`once_cell`](once_cell.md) or `std`'s `OnceLock` is the better fit, since it
hands back the value instead of leaving you to store it yourself.

#### `ReentrantMutex`

A mutex the same thread may lock more than once.

```
use parking_lot::ReentrantMutex;

let lock = ReentrantMutex::new(5u32);

let outer = lock.lock();
let inner = lock.lock(); // <- same thread, so this does not deadlock

assert_eq!(*outer, 5);
assert_eq!(*inner, 5);
```

**When to use it:** reluctantly. It exists for recursive call paths and for FFI
callbacks that re-enter your code, and it hands out shared references only —
there is no `&mut`, because several guards can be alive at once. A design that
needs it is usually a design worth revisiting; `std` has no equivalent partly
for that reason.

#### `FairMutex`

Hands the lock to the longest waiter rather than whoever gets there first.

```
use parking_lot::FairMutex;

let lock = FairMutex::new(0u32);
*lock.lock() += 1;

assert_eq!(*lock.lock(), 1);
```

**When to use it:** when a thread is being starved — a slow consumer that never
wins against fast ones. Fairness costs throughput, because handing the lock
directly to a waiting thread prevents the fast path where a thread reacquires a
lock it just released. Reach for it after observing starvation, not before.

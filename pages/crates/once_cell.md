---
title: "once_cell"
version: "1.21.4"
publisher: "Alex Kladov (matklad), Michal 'vorner' Vaner (vorner)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-02"
summary: "Values initialised once, on first use: a cell you can fill later and a lazy static that runs its initialiser at first access. Most of it is now in `std` — this is the version for older toolchains, `no_std`, and the parts `std` still lacks."
categories: ["memory-management", "concurrency", "no-std"]
repository: "https://github.com/matklad/once_cell"
---

## Overview

A global that needs computing — a compiled regex, a parsed config, a connection
pool — has nowhere obvious to live. `static` demands a value known at compile
time, so anything needing allocation or a function call is out.

`once_cell` provides the two pieces that solve it: a **cell** you can fill
exactly once and then read forever, and a **lazy** wrapper that runs an
initialiser the first time anyone looks.

```
use once_cell::sync::Lazy;
use std::collections::HashMap;

static SETTINGS: Lazy<HashMap<&str, u32>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("timeout", 30);
    m
});

assert_eq!(SETTINGS["timeout"], 30); // <- built here, on first access
assert_eq!(SETTINGS.len(), 1);       // <- and not again
```

**Read this before adding the dependency: `std` now has all of this.** The
design was adopted wholesale — `OnceLock` and `OnceCell` stabilised in Rust
1.70, `LazyLock` and `LazyCell` in 1.80. For new code on a current toolchain,
the standard library is the better answer, and this crate's own documentation
says so.

- `sync::OnceCell` became `std::sync::OnceLock` (1.70).
- `unsync::OnceCell` became `std::cell::OnceCell` (1.70).
- `sync::Lazy` became `std::sync::LazyLock` (1.80).
- `unsync::Lazy` became `std::cell::LazyCell` (1.80).

So what is it still for?

- **An MSRV below those releases.** A library supporting older compilers cannot
  use the `std` types at all.
- **`no_std`.** The `race` module offers lock-free cells for targets with no
  operating system, which `std` by definition cannot.
- **The API `std` hasn't taken.** `get_or_try_init` — a fallible initialiser
  that leaves the cell empty on error — is the one people miss most, along with
  `wait`, which blocks until another thread initialises.

It is not deprecated or unmaintained: it is actively released, and remains a
dependency of a large part of the ecosystem because so many crates support older
compilers. But if you are writing an application on a recent toolchain, reach
for `std` and skip this.

The other thing worth knowing is **`sync` versus `unsync`**. The `sync` types are
thread-safe and cost an atomic check per access; `unsync` is single-threaded and
cheaper, and won't compile in a `static`. Pick `unsync` only when the value is
genuinely thread-local. It requires Rust 1.65 and has no required dependencies.

## When to use it

### Use case: A global that has to be computed

The classic case, and the one `lazy_static!` used to serve. A regex compiled
once at first use, not on every call.

```
use once_cell::sync::Lazy;

// Stands in for something expensive — a Regex, a parsed schema, a pool.
static EXPENSIVE: Lazy<Vec<u32>> = Lazy::new(|| (1..=1000).collect());

fn contains(n: u32) -> bool {
    EXPENSIVE.contains(&n)
}

assert!(contains(500));
assert!(!contains(2000));
assert_eq!(EXPENSIVE.len(), 1000);
```

**Why it fits:** the cost is paid once, on the first call, and every later access
is a pointer dereference behind one atomic check. Compared with `lazy_static!`
the type is ordinary — `Lazy<T>` derefs to `T` rather than being a generated
macro type, so it shows up properly in editors and error messages.

### Use case: A value supplied later, at startup

Configuration read from the environment, or a handle created after `main`
begins, isn't available where the `static` is written. `OnceCell` separates
declaring the slot from filling it.

```
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<String> = OnceCell::new();

fn init(value: &str) -> Result<(), String> {
    CONFIG.set(value.to_string()).map_err(|_| "already initialised".to_string())
}

fn endpoint() -> &'static str {
    CONFIG.get().map(String::as_str).unwrap_or("unset")
}

assert_eq!(endpoint(), "unset"); // <- before init
init("https://api.example.com").unwrap();
assert_eq!(endpoint(), "https://api.example.com");

// Setting twice is an error, not a silent overwrite.
assert!(init("https://other").is_err());
```

**Why it fits:** the value is immutable once set, so readers need no lock, and a
second `set` fails loudly rather than swapping the value under code that already
read it. Reading before initialisation is a visible `None` rather than a panic.

### Use case: An initialiser that can fail

Anything reading a file or opening a connection can fail, and a lazy static has
nowhere to report that — the closure must produce a value or panic.
`get_or_try_init` gives the error back and leaves the cell empty to retry.

```
use once_cell::sync::OnceCell;

static PARSED: OnceCell<u32> = OnceCell::new();

fn parsed(source: &str) -> Result<u32, std::num::ParseIntError> {
    PARSED.get_or_try_init(|| source.parse::<u32>()).copied()
}

// A failure leaves the cell empty.
assert!(parsed("not a number").is_err());
assert!(PARSED.get().is_none());

// So a later attempt can still succeed.
assert_eq!(parsed("42").unwrap(), 42);
assert_eq!(PARSED.get(), Some(&42));
```

**Why it fits:** this is the API `std` does not yet have, and the reason to keep
the dependency on a modern toolchain. `LazyLock` would have to panic here, which
turns a missing config file into a crash with no useful location.

## API map

The crate is two modules of two types. `sync` is thread-safe and the one you
want for a `static`; `unsync` is the cheaper single-threaded version. Both
expose the same shape, so the entries below use `sync` throughout.

### `OnceCell`

#### `OnceCell::new` and `set`

An empty cell, filled at most once.

```
use once_cell::sync::OnceCell;

let cell: OnceCell<u32> = OnceCell::new();

assert!(cell.get().is_none());
assert_eq!(cell.set(7), Ok(()));
assert_eq!(cell.get(), Some(&7));

// The second set fails and hands the value back.
assert_eq!(cell.set(9), Err(9));
assert_eq!(cell.get(), Some(&7));
```

**When to use it:** when the value arrives from somewhere the declaration can't
reach — command-line arguments, a config file, a handle built in `main`. `new`
is `const`, which is what lets it be a `static`.

#### `get_or_init`

Returns the value, running the initialiser if the cell is empty.

```
use once_cell::sync::OnceCell;

let cell: OnceCell<String> = OnceCell::new();

let first = cell.get_or_init(|| "computed".to_string());
assert_eq!(first, "computed");

// The closure does not run again.
let second = cell.get_or_init(|| panic!("never called"));
assert_eq!(second, "computed");
```

**When to use it:** lazy initialisation where you hold the cell rather than a
`static` — a field on a struct caching something derived. Exactly one closure
wins under contention; the others' results are discarded, so it must be free of
side effects you care about.

#### `get_or_try_init`

The fallible form: an `Err` leaves the cell empty.

```
use once_cell::sync::OnceCell;

let cell: OnceCell<u32> = OnceCell::new();

let failed: Result<&u32, &str> = cell.get_or_try_init(|| Err("nope"));
assert!(failed.is_err());
assert!(cell.get().is_none()); // <- still empty, so retryable

let ok: Result<&u32, &str> = cell.get_or_try_init(|| Ok(5));
assert_eq!(ok, Ok(&5));
```

**When to use it:** initialisation that reads a file, opens a socket or parses
input. This is the main API `std`'s `OnceLock` still lacks, and the usual reason
to keep this crate on a current toolchain.

#### `get` and `get_mut`

Reads without initialising.

```
use once_cell::sync::OnceCell;

let mut cell: OnceCell<u32> = OnceCell::new();
assert_eq!(cell.get(), None);

cell.set(1).unwrap();
if let Some(v) = cell.get_mut() {
    *v += 1; // <- needs &mut, so no other reader can exist
}
assert_eq!(cell.get(), Some(&2));
```

**When to use it:** `get` to check whether initialisation has happened without
triggering it. `get_mut` is the one exception to immutability — it requires
`&mut self`, so the borrow checker guarantees exclusivity, which is why it is
sound.

#### `wait`

Blocks until another thread initialises the cell.

```
use once_cell::sync::OnceCell;
use std::sync::Arc;

let cell: Arc<OnceCell<u32>> = Arc::new(OnceCell::new());
let writer = Arc::clone(&cell);

let handle = std::thread::spawn(move || {
    writer.set(99).unwrap();
});

// Blocks until the other thread has set it.
assert_eq!(*cell.wait(), 99);
handle.join().unwrap();
```

**When to use it:** a startup handshake where one thread produces a value others
need before proceeding. It is a `sync`-only method with no `std` equivalent, and
it deadlocks if nobody ever sets the cell — so it belongs in code where the
writer is guaranteed to run.

#### `take` and `into_inner`

Recover the value, emptying the cell.

```
use once_cell::sync::OnceCell;

let mut cell = OnceCell::new();
cell.set("value".to_string()).unwrap();

assert_eq!(cell.take(), Some("value".to_string()));
assert!(cell.get().is_none()); // <- reusable

let other: OnceCell<u8> = OnceCell::from(3);
assert_eq!(other.into_inner(), Some(3));
```

**When to use it:** reclaiming an owned value at shutdown, or resetting a cell in
a test between cases. Both need ownership or `&mut`, so neither can pull the
value out from under a reader.

### `Lazy`

#### `Lazy::new`

Pairs a cell with its initialiser, so no separate `get_or_init` call is needed.

```
use once_cell::sync::Lazy;

static NUMBERS: Lazy<Vec<u32>> = Lazy::new(|| vec![1, 2, 3]);

// Deref makes it behave like the inner value.
assert_eq!(NUMBERS.len(), 3);
assert_eq!(NUMBERS[0], 1);
assert_eq!(NUMBERS.iter().sum::<u32>(), 6);
```

**When to use it:** the default for a computed global. The `Deref` impl is what
makes it pleasant — you write `NUMBERS.len()`, not `NUMBERS.get().len()` — and
it is the direct replacement for `lazy_static!`, with a real type instead of a
macro-generated one.

#### `Lazy::force`

Runs the initialiser explicitly, without going through `Deref`.

```
use once_cell::sync::Lazy;

let lazy: Lazy<u32> = Lazy::new(|| 42);

assert_eq!(Lazy::get(&lazy), None); // <- not yet initialised
assert_eq!(*Lazy::force(&lazy), 42);
assert_eq!(Lazy::get(&lazy), Some(&42));
```

**When to use it:** paying an initialisation cost at a chosen moment — warming a
cache during startup rather than on the first request that needs it. Note the
call style: `Lazy::force(&lazy)` rather than a method, so it cannot collide with
a method of the same name on the inner type.

### Single-threaded and `no_std`

#### `unsync::Lazy` and `unsync::OnceCell`

The same API without the atomics.

```
use once_cell::unsync::{Lazy, OnceCell};

let cell: OnceCell<u32> = OnceCell::new();
cell.set(1).unwrap();
assert_eq!(cell.get(), Some(&1));

let lazy: Lazy<String> = Lazy::new(|| "built".to_string());
assert_eq!(&*lazy, "built");
```

**When to use it:** a cache inside a struct that never crosses threads — a
memoised field, a parser's interner. It is neither `Send` nor `Sync`, so it
cannot be a `static`; the compiler tells you so, which is the safety net that
makes choosing `unsync` low-risk.

#### `race::OnceBox`

A lock-free cell for `no_std`, where losing an initialisation race is acceptable.

```
use once_cell::race::OnceBox;

let cell: OnceBox<u32> = OnceBox::new();

assert!(cell.get().is_none());
cell.set(Box::new(7)).unwrap();
assert_eq!(cell.get(), Some(&7));

// Like OnceCell, a second set is refused.
assert!(cell.set(Box::new(9)).is_err());
```

**When to use it:** embedded targets and other `no_std` contexts, where blocking
is not available. The trade is in the name — if two threads initialise at once
both closures run and one result is thrown away, whereas `sync::OnceCell` blocks
the loser. `race` also offers `OnceBool` and `OnceNonZeroUsize` for values that
fit in an atomic, needing no allocation at all.

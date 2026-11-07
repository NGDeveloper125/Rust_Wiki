---
title: "lazy_static"
version: "1.5.1"
publisher: "rust-lang-owner"
publisher_url: "https://crates.io/users/rust-lang-owner"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-07"
summary: "Superseded by `std::sync::LazyLock`, and the repository is archived. The macro that gave Rust lazily initialised statics for a decade — documented here for the code that still depends on it, and for how to move off it."
domain: "Async runtimes & concurrency"
categories: ["memory-management", "no-std", "rust-patterns", "unmaintained"]
repository: "https://github.com/rust-lang-nursery/lazy-static.rs"
docs: "https://docs.rs/lazy_static/1.5.1/lazy_static/"
---

## Overview

**Superseded.** The repository is archived, and 1.5.1 — published 2026-10-01 —
exists only to say so: it changed the README and the crate docs to read *"this
crate has been replaced by the `LazyLock` type, which is stable since Rust
1.80.0 with a slightly different syntax. Barring breakages and security fixes,
this crate will no longer be updated."* Nothing else differs between 1.5.0 and
1.5.1, and no `#[deprecated]` attribute was added, so a build that depends on it
keeps compiling without a word of warning. **Use instead:**
[`std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
on Rust 1.80 or newer, or [`once_cell`](once_cell.md) below that. One case is
still not covered by either — a lazy [`static`](../syntax/keywords/static.md) in
a `no_std` binary — and it is the last use case on this page.

That is the whole recommendation. The rest of this page exists because the crate
has been downloaded over a billion times and is still pulled in roughly 250
million times every ninety days, which means a great deal of code was written
against it and somebody has to maintain that code.

What it does: a `static` must be initialised with a value the compiler can
compute, so anything that allocates or calls a function is out. `lazy_static!`
works around that by generating, for each declaration, a private zero-sized type
that implements `Deref`; the first dereference runs your expression and stores
the result, and every later one hands back a reference to it. From 2014 until
Rust 1.80 that was the only ergonomic way to have a global
[`HashMap`](../concepts/collections-strings/hashmap-and-hashset.md), a compiled
[`Regex`](regex.md) or a connection pool.

It is a [declarative macro](../concepts/macros-metaprogramming/declarative-macros.md)
and nothing more. There are **no dependencies** by default and the compiled
weight is a hidden `Once` plus a `MaybeUninit` per declaration — the crate was
never a cost worth arguing about, and that is not why it was replaced. It was
replaced because the macro has to invent a type to hang `Deref` on, and an
invented type is the source of every complaint about it: it does not appear in
rustdoc as a value, it cannot be named, stored in a struct, or passed to a
generic function, and the compiler errors talk about a type you did not write.
`LazyLock<T>` and `once_cell`'s `Lazy<T>` are ordinary types, which removes all
of that at once.

Minimum supported Rust is **1.40**, which the README commits to and which is
lower than any replacement — `LazyLock` needs 1.80, and `once_cell` 1.21 declares
1.65. On a toolchain older than that, this crate is still the answer.

## When to use it

For new code on a current toolchain, don't — the three situations below are the
ones that actually turn up: reading code that uses it, getting code off it, and
the single niche where nothing has replaced it yet.

### Use case: Reading the code you have inherited

The shape you will find in any project started before 2024. A lookup table and
a derived value, both built on first use.

```
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref CODES: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(404, "not found");
        m.insert(500, "server error");
        m
    };
    // One lazy static may read another in its initialiser.
    static ref COUNT: usize = CODES.len();
    static ref DEFAULT: String = describe(400);
}

fn describe(code: u32) -> String {
    match CODES.get(&code) {
        Some(text) => format!("{code} {text}"),
        None => format!("{code} unknown"),
    }
}

assert_eq!(*COUNT, 2);             // <- the `*` is not optional
assert_eq!(CODES.len(), 2);        // <- but method calls autoderef
assert_eq!(*DEFAULT, "400 unknown");
assert_eq!(describe(404), "404 not found");
```

**Why it fits:** nothing here needs changing to keep working, and the two access
styles are the thing to recognise. `*COUNT` is a dereference of the generated
type; `CODES.len()` reaches the `HashMap` through the same `Deref` without
saying so. Where a reader trips is the gap between them — `CODES.len()` compiles
and `CODES == other_map` does not, for reasons the next use case makes concrete.

### Use case: Moving a module off the crate

The translation is mechanical, which is the good news: `static ref NAME: T = e;`
becomes `static NAME: LazyLock<T> = LazyLock::new(|| e);`, and
`lazy_static::initialize(&NAME)` becomes `LazyLock::force(&NAME)`. Dereferencing
is unchanged, so call sites usually compile untouched.

```
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::LazyLock;

fn build() -> HashMap<&'static str, u32> {
    HashMap::from([("timeout", 30), ("retries", 3)])
}

// Before.
lazy_static! {
    static ref OLD: HashMap<&'static str, u32> = build();
}

// After. The initialiser is a closure, so `LazyLock::new` can be `const`.
static NEW: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(build);

// Forcing initialisation early, before and after.
lazy_static::initialize(&OLD);
let _: &HashMap<_, _> = LazyLock::force(&NEW);

// Reading is identical, and the values are identical.
assert_eq!(OLD["timeout"], 30);
assert_eq!(NEW["timeout"], 30);
assert_eq!(*OLD, *NEW);

// What you gain: `LazyLock<T>` is a type with a name, so it composes.
struct Config {
    table: &'static LazyLock<HashMap<&'static str, u32>>,
}
fn timeout(t: &LazyLock<HashMap<&'static str, u32>>) -> u32 { t["timeout"] }

assert_eq!(timeout(Config { table: &NEW }.table), 30);
// None of those three lines can be written against `OLD`: there is no type to
// write down. `std::mem::size_of::<OLD>()` is 0 -- it is a marker, and the
// storage lives in a separate hidden static the macro generated beside it.
assert_eq!(std::mem::size_of::<OLD>(), 0);
```

**Why it fits:** the mechanical half is safe to do in bulk. Two things to watch.
First, `LazyLock` takes one `static` per value where `lazy_static!` took a block,
so a block of six becomes six statics — tedious, not difficult. Second, anything
relying on the macro's type will break, and that is the part a mass rename will
not catch: attributes attach to the *generated* type, not your value, so a
`#[derive(Debug)]` on a `lazy_static!` item prints `CODES { __private_field: () }`
rather than the map. After the move, `{:?}` starts printing the value — which is
what the author of that line wanted in the first place.

### Use case: A lazy static on bare metal

This is the one place the crate has not been replaced. `LazyLock` is `std`-only,
and `core` offers just `LazyCell`, which is not `Sync` and so cannot be a
`static`. `lazy_static` has a feature for it: enabling `spin_no_std` swaps the
`std::sync::Once` inside for `spin::Once`, which spins on an atomic instead of
parking the thread.

```
// With `features = ["spin_no_std"]` in Cargo.toml, this compiles for a
// `#![no_std]` target such as thumbv7m-none-eabi. Without it, the default
// backend pulls in `std` and the build fails with E0463, `can't find crate
// for std` -- `default-features = false` on its own changes nothing, because
// the std backend is the no-feature path rather than a default feature.
use lazy_static::lazy_static;

lazy_static! {
    static ref TABLE: [u16; 8] = {
        let mut t = [0u16; 8];
        let mut i = 0;
        while i < 8 {
            t[i] = (i as u16) * 3;
            i += 1;
        }
        t
    };
}

assert_eq!(TABLE[3], 9);
assert_eq!(TABLE.len(), 8);
```

**Why it fits:** it needs nothing from the platform but atomics. The maintained
alternative is `once_cell` with `default-features = false` and
`features = ["critical-section"]` — with no features at all its `sync` module
does not exist, and the import fails with E0432 — but that then wants a
`critical-section` implementation crate for your chip, so it is a larger
commitment than a busy-wait. Weigh them honestly: `spin_no_std` is the lighter
option and it is on a crate that will not be fixed again.

## API map

The entire public surface is one macro, one free function, and one trait that
exists to support the function. Everything else is the shape of the code the
macro writes for you, which is what most of these entries are about.

### Declaring a lazy static

#### `lazy_static!`

The only macro. Each declaration is `static ref NAME: TYPE = EXPR;`.

```
use lazy_static::lazy_static;

lazy_static! {
    static ref GREETING: String = {
        let who = "world";
        format!("hello {who}")
    };
}

assert_eq!(*GREETING, "hello world");
```

**When to use it:** only in code that already has it. For anything new, the same
line as `static GREETING: LazyLock<String> = LazyLock::new(|| ...)` — the
replacement is strictly more capable and in the standard library. The one reason
to reach for the macro deliberately is a project pinned below Rust 1.65, which
is the floor `once_cell` now declares.

#### Several declarations in one block

One invocation takes any number of statics, and an initialiser may read an
earlier one.

```
use lazy_static::lazy_static;

lazy_static! {
    static ref BASE: Vec<u32> = (1..=4).collect();
    static ref TOTAL: u32 = BASE.iter().sum();  // <- reads BASE
    static ref LABEL: String = format!("{} items, sum {}", BASE.len(), *TOTAL);
}

assert_eq!(*TOTAL, 10);
assert_eq!(*LABEL, "4 items, sum 10");
```

**When to use it:** to group statics that belong together — and note that
reading one from another is fine *as long as the dependencies form a tree*. A
cycle, where `A`'s initialiser reads `B` and `B`'s reads `A`, re-enters the same
`Once` on one thread; the crate's own docs warn it deadlocks, and `std` leaves
the behaviour of a recursive `call_once` unspecified. This is the one
`lazy_static!` hazard with no compile-time check behind it, and it does not go
away when you migrate — `LazyLock` has the same problem.

#### Visibility: `pub`, `pub(crate)`, `pub(in path)`

The visibility goes before `static`, and all the usual
[`pub`](../syntax/keywords/pub.md) forms are accepted.

```
mod config {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref NAME: &'static str = "service";
        pub(crate) static ref PORT: u16 = 8080;
        pub(in crate::config) static ref SECRET: &'static str = "internal";
        static ref PRIVATE: u8 = 1;
    }

    pub fn describe() -> String {
        format!("{} {} {} {}", *NAME, *PORT, *SECRET, *PRIVATE)
    }
}

fn main() {
    assert_eq!(*config::NAME, "service");
    assert_eq!(*config::PORT, 8080);
    assert_eq!(config::describe(), "service 8080 internal 1");
}
```

**When to use it:** as you would on any item. Be aware of what a reader of your
rustdoc sees: the macro marks the generated static `#[doc(hidden)]` and the
visibility lands on the generated *struct*, so a `pub static ref` is published
as an undocumented unit struct rather than as a value. If a public global is part
of your API, that alone is a reason to move it to `LazyLock`.

#### Attributes and doc comments

Attributes are accepted ahead of each declaration and attach to the generated
type.

```
use lazy_static::lazy_static;

lazy_static! {
    /// Doc comments work, and land on the generated struct.
    #[derive(Debug)]
    static ref TAGS: Vec<&'static str> = vec!["a", "b"];
}

// Deriving Debug does not do what it looks like it does:
assert_eq!(format!("{TAGS:?}"), "TAGS { __private_field: () }");
// The value needs an explicit deref.
assert_eq!(format!("{:?}", *TAGS), r#"["a", "b"]"#);
```

**When to use it:** for doc comments, freely. For derives, understand that you
are deriving on a zero-sized marker — the line above is the single most
confusing thing the macro does, and it compiles silently. And keep
[`#[cfg]`](../syntax/attributes/cfg-attribute.md) *outside* the block: the
attribute is forwarded only to the struct, not to the companion static or the
`Deref` impl, so a `#[cfg]`-ed declaration that compiles out fails with E0573,
`expected type, found static`. Put it on the whole invocation instead —
`#[cfg(unix)] lazy_static! { ... }` — which works correctly.

### Reading the value

#### `*NAME` and deref coercion

The generated type derefs to yours, so the value is reachable three ways.

```
use lazy_static::lazy_static;

lazy_static! {
    static ref WORDS: Vec<String> = vec!["one".into(), "two".into()];
}

fn count(v: &Vec<String>) -> usize { v.len() }

assert_eq!(WORDS.len(), 2);          // method call: autoderef
assert_eq!(count(&WORDS), 2);        // argument: deref coercion
assert_eq!((*WORDS)[0], "one");      // explicit deref
assert_eq!(WORDS.iter().count(), 2); // iterator, via a method
```

**When to use it:** prefer the explicit `*NAME` when the line is about the value
itself (a comparison, a format argument, passing it to a generic). Method calls
read better without it. The underlying mechanism is
[deref coercion](../concepts/ownership-borrowing/deref-coercion.md) and it has
the limits that implies — covered next.

#### What the generated type is not

It is not your type, and `Deref` only stretches so far.

```
// no-compile
use lazy_static::lazy_static;

lazy_static! {
    static ref NAMES: Vec<String> = vec!["a".to_string()];
}

fn generic<T: AsRef<[String]>>(x: T) -> usize { x.as_ref().len() }

fn main() {
    generic(&NAMES);                      // E0277: AsRef not implemented for NAMES
    let _ = NAMES == vec!["a".to_string()]; // E0369: `==` cannot be applied
    for _n in &NAMES {}                   // E0277: `&NAMES` is not an iterator
}
```

**When to use it:** as a lookup when one of those three errors names a type you
never defined. The fix is always a `*` or `&*` — `generic(&*NAMES)`,
`*NAMES == v`, `for n in NAMES.iter()` — because coercion applies to a method
receiver and a concrete argument type, never to a trait bound, an operator, or
`IntoIterator`. This is
[deref polymorphism](../concepts/design-patterns-idioms/anti-pattern-deref-polymorphism.md)
hitting its ceiling, and it is why the standard library's version is a real
generic type rather than a generated one.

#### Getting a `&'static T`

Because the static outlives everything, the reference out of it does too.

```
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref LOOKUP: HashMap<&'static str, u32> =
        HashMap::from([("a", 1), ("b", 2)]);
}

// The deref borrows from a `static`, so the result is good for 'static.
fn table() -> &'static HashMap<&'static str, u32> { &*LOOKUP }

fn first_key() -> &'static str {
    // Values borrowed out of it are 'static as well.
    table().keys().copied().min().unwrap()
}

assert_eq!(table().len(), 2);
assert_eq!(first_key(), "a");
```

**When to use it:** when a function has to hand back a borrow and you do not
want it tied to a parameter — a parser returning slices of a static keyword
table, say. It is the one ergonomic edge the macro has over
`once_cell::sync::Lazy`, where you would reach for `Lazy::force` to get the same
`&'static`; `LazyLock::force` is the equivalent.

### Controlling initialisation

#### `initialize`

Runs the initialiser now, rather than at the first unpredictable access.

```
use lazy_static::lazy_static;

lazy_static! {
    static ref BUFFER: Vec<u8> = (0..=255).collect();
}

fn main() {
    // Pay the cost here, during startup, where it can be measured.
    lazy_static::initialize(&BUFFER);
    assert_eq!(BUFFER.len(), 256);
}
```

**When to use it:** when *when* the work happens matters — a latency budget that
must not absorb a first-request penalty, or an initialiser that can panic and
should bring the process down at startup instead of inside a request handler.
Calling it twice is harmless. Its replacement is `LazyLock::force(&X)`, which
differs in returning the `&T` rather than `()`.

#### `LazyStatic`

The trait that makes `initialize` generic. Implemented for every declaration.

```
use lazy_static::{lazy_static, LazyStatic};

lazy_static! {
    static ref A: u32 = 1;
    static ref B: String = "two".to_string();
}

// A bound you can write yourself, to force a set of statics in one place.
fn warm<T: LazyStatic>(items: &T) { lazy_static::initialize(items); }

warm(&A);
warm(&B);
assert_eq!(*A, 1);
assert_eq!(*B, "two");
```

**When to use it:** rarely and deliberately — as a bound on your own startup
helper. Its single method is `#[doc(hidden)]`, so the trait is only useful for
naming the set of lazy statics; call `initialize` to do the work. There is no
equivalent for `LazyLock`, which has no trait, so a helper like `warm` becomes a
macro or a plain list of `force` calls after migrating.

#### A panicking initialiser poisons the static

If the expression panics, the value is not retried — the `Once` behind it is
poisoned and every later access panics too.

```
use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: String = panic!("config missing");
}

// The first access reports the real cause.
let first = std::panic::catch_unwind(|| CONFIG.len());
assert!(first.is_err());

// Every access after that reports only the poisoning.
let second = std::panic::catch_unwind(|| CONFIG.len());
assert!(second.is_err()); // <- panics with "Once instance has previously been poisoned"
```

**When to use it:** to decide where a fallible initialiser belongs. This is the
reason not to put `unwrap` in a `lazy_static!` block for anything that can
genuinely fail: the useful error appears once, on whichever thread touched it
first, and every later report is the unhelpful second message. Build the value
in `main` and pass it down, or use a cell with a fallible getter —
`once_cell`'s `get_or_try_init` is the direct tool for it. Under `spin_no_std`
the same thing happens with a different message, `Once panicked`.

### Constraints on the value

#### `T` must be `Sync`

A lazy static is a `static`, so the type has to be shareable across threads.

```
// no-compile
use lazy_static::lazy_static;
use std::cell::RefCell;

lazy_static! {
    // E0277: `RefCell<u32>` cannot be shared between threads safely,
    // from the `T: Sync` bound on the crate's internal `Lazy<T>`.
    static ref COUNTER: RefCell<u32> = RefCell::new(0);
}

fn main() {}
```

**When to use it:** as the explanation when that error points into the macro.
The answer is the same as for any global that needs mutation: a `Mutex` or
`RwLock` ([`parking_lot`](parking_lot.md)'s or `std`'s) rather than a `Cell`,
or an atomic if the value is a number. See
[`Send` and `Sync`](../concepts/concurrency-async/send-and-sync.md). The
initialiser itself runs exactly once no matter how many threads arrive together
— that part the crate guarantees.

#### Destructors never run

Like any `static`, the value is never dropped.

```
use lazy_static::lazy_static;

struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) { println!("dropped {}", self.0); }
}

lazy_static! {
    static ref GLOBAL: Noisy = Noisy("global");
}

fn main() {
    let _ = GLOBAL.0;
    let local = Noisy("local");
    assert_eq!(local.0, "local");
    // At exit, "dropped local" is printed. "dropped global" is not, ever.
}
```

**When to use it:** never put a thing that must be released in one. A file
handle, a lock file, a span exporter with buffered output, a database connection
that should say goodbye — all of them leak, and the leak is silent. Flush such
things explicitly before returning from `main`. This is not a defect of the
crate and migrating does not fix it: `LazyLock` in a `static` behaves the same
way, because the rule belongs to `static`, not to the wrapper.

### Running without std

#### The `spin_no_std` feature

The crate's one feature. It replaces `std::sync::Once` with `spin::Once`.

```
// Cargo.toml needs the feature named explicitly:
//   lazy_static = { version = "1.5.1", features = ["spin_no_std"] }
// It adds exactly one dependency, `spin`, and is the only way to build the
// crate for a target without std.
use lazy_static::lazy_static;

lazy_static! {
    static ref MASKS: [u8; 4] = [1, 2, 4, 8];
}

assert_eq!(MASKS[2], 4);
```

**When to use it:** on a `no_std` target, where it is the only option the crate
offers — and where, unusually for this page, the crate is still a defensible
choice, since `LazyLock` does not exist without `std`. Two things to know. It is
not a `default-features = false` situation: the std backend is selected by the
*absence* of the feature, so you must add `spin_no_std` for a bare-metal build
or you get E0463. And `spin::Once` busy-waits where `std::sync::Once` parks the
thread, so a contended first initialisation burns cycles instead of yielding —
on a microcontroller that is usually what you want anyway.

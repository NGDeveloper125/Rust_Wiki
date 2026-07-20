---
title: "Constructor functions (new() convention)"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Builders & Object Construction"]
related_syntax: [fn]
see_also: ["The Default trait as idiom", "Privacy for extensibility", "Structs", "Visibility & privacy (pub and friends)"]
---

## Explanation

Rust has no `new` keyword and no built-in notion of a constructor — a
type is just data, and producing one is just a function call like any
other. The convention that fills this gap is a plain associated function
named `new`, defined in the type's own `impl` block, that builds and
returns an instance of `Self`. Because it's ordinary code rather than
compiler-recognized syntax, nothing stops a type from having several
constructors side by side (`new`, `with_capacity`, `from_bytes`), each
named for what it does rather than overloaded under one reserved word —
something languages with a real `new` keyword and constructor overloading
can't offer as cleanly.

The convention carries real expectations, not just a naming habit. A
function called `new` is expected to be infallible and non-consuming: it
takes whatever inputs it needs and always succeeds in producing a value,
with no `Result` and no borrowed argument it fails to give back. When
construction can fail — a port number that's out of range, a string that
isn't valid UTF-8 — the idiomatic move is a differently-named function
returning `Result<Self, E>` (`Config::from_file`, `Port::try_new`), so the
fallibility is visible in both the name and the signature rather than
hidden behind a name that promises "this always works."

`new` also pairs naturally with keeping a type's fields private (see
[Privacy for extensibility](privacy-for-extensibility.md)): once outside
code can't build the type with a struct literal, `new` (or whichever
constructor a type exposes) becomes the *only* path to an instance, which
means it's also the only place that has to enforce the type's invariants.
Everything downstream of construction can then simply trust that an
existing value is valid, because there was never a way to build an
invalid one.

Where a type genuinely has no required inputs, [the `Default`
trait](the-default-trait-as-idiom.md) often takes over `new`'s job
instead of duplicating it — the two idioms are closely related and the
API Guidelines recommend implementing both when a zero-argument `new`
exists, with `new` delegating to `Default::default()` (or vice versa) so
the two never drift apart.

## Basic usage example

```
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u64,
}

impl RetryPolicy {
    fn new(max_attempts: u32, backoff_ms: u64) -> Self { // <- constructor: builds and returns Self, always succeeds
        RetryPolicy { max_attempts, backoff_ms }
    }
}

let policy = RetryPolicy::new(3, 250);
println!("{} attempts", policy.max_attempts);
```

## Best practices & deeper information

### Scenario: Creating a new object

A connection pool has one truly required input — how many connections to
keep open — so `new` takes exactly that and nothing else, leaving room
for a separate constructor later if more configuration knobs show up.

```
struct ConnectionPool {
    size: usize,
    connections: Vec<u32>, // stand-in for real connection handles
}

impl ConnectionPool {
    fn new(size: usize) -> Self { // <- new(): the minimal, infallible constructor
        ConnectionPool {
            size,
            connections: (0..size as u32).collect(),
        }
    }
}

let pool = ConnectionPool::new(8);
println!("pool has {} connections", pool.connections.len());
```

**Why this way:** `new` taking only the genuinely required parameters and
always succeeding is the convention readers expect from the name itself;
the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/ctor.html)
book documents this exact shape as the idiomatic Rust constructor.

### Scenario: Designing a public API

A parser can be built from different sources — a file path, a raw
string, a set of already-parsed tokens — so instead of overloading one
`new`, each source gets its own clearly-named constructor.

```
struct Config {
    entries: Vec<(String, String)>,
}

impl Config {
    fn new() -> Self { // <- new(): the empty, always-valid starting point
        Config { entries: Vec::new() }
    }

    fn from_str(source: &str) -> Self { // <- named for its source, not overloaded under `new`
        let entries = source
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config { entries }
    }
}

let empty = Config::new();
let loaded = Config::from_str("host=localhost\nport=8080");
println!("{} {}", empty.entries.len(), loaded.entries.len());
```

**Why this way:** without constructor overloading, giving each
construction path its own name keeps every one of them self-documenting
at the call site — the
[API Guidelines' constructor conventions](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
recommend exactly this: static, inherent methods named for what they
build from, with `new` reserved for the no-argument-or-obvious-argument
case.

## Embedded Rust Notes

**Full support.** `new()` is an ordinary associated function with no
special compiler support — it works identically under `#![no_std]`,
including on targets with no allocator, as long as the constructor itself
doesn't require one (a `RetryPolicy` or `ConnectionPool`-style value made
of plain fields costs nothing beyond the fields themselves).

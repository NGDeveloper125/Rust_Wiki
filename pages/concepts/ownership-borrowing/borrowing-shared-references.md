---
title: "Borrowing (shared references)"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["&"]
see_also: ["Ownership", "Mutable borrowing", "The borrow checker"]
---

## Explanation

Borrowing lets code access a value without taking ownership of it. A
shared reference (`&T`) grants read-only access for a limited scope,
after which the original owner remains fully in control — nothing about
ownership changes, and the value is not moved or copied.

This solves a problem ownership alone creates: if passing a value to a
function always moved it, you'd need to pass it back out again (or clone
it) just to keep using it afterward. Borrowing lets a function (or any
other piece of code) *use* a value temporarily without the caller losing
access to it.

Any number of shared references to the same value can exist
simultaneously — this is safe precisely because a shared reference cannot
mutate through it (unless the type uses
[interior mutability](interior-mutability.md); see also
[Immutability by default](immutability-by-default.md)).
The tradeoff for that safety is a lifetime constraint: a reference can
never outlive the value it points to, which the compiler verifies
statically (see [The borrow checker](borrow-checker.md) and
[Lifetimes](lifetimes.md)) rather than checking at runtime the way a
garbage-collected language would.

## Basic usage example

```
let s = String::from("hello");
let r1 = &s;
let r2 = &s; // <- a second shared reference coexists safely with r1

println!("{r1} and {r2}");
println!("{s}"); // s is still usable: borrowing never took ownership
```

**Restriction:** a shared reference only permits reading — mutating
through it (unless the type uses interior mutability), or mutating the
original value while any shared reference to
it is still alive, is rejected at compile time.

## Best practices & deeper information

### Scenario: Sharing data with multiple references

A report function reads two different fields of the same struct through
separate shared borrows at once — safe because neither borrow grants
write access.

```
struct Inventory {
    in_stock: Vec<String>,
    reserved: Vec<String>,
}

fn report(stock: &[String], reserved: &[String]) {
    println!("{} in stock, {} reserved", stock.len(), reserved.len());
}

let inv = Inventory {
    in_stock: vec!["widget".into()],
    reserved: vec!["gadget".into()],
};

report(&inv.in_stock, &inv.reserved); // <- two live shared borrows of `inv`, both read-only
```

**Why this way:** because `&T` never grants write access, any number of
shared borrows of the same or overlapping data can coexist safely — the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
calls this out as the reason reading is unrestricted while writing stays
exclusive.

### Scenario: Multi-threading

Several worker threads need read-only access to the same local data
without moving it into each thread or reaching for `Arc` — `thread::scope`
lets them borrow it directly.

```
use std::thread;

let samples = vec![10, 20, 30, 40];

thread::scope(|s| {
    for chunk in samples.chunks(2) {
        s.spawn(move || { // <- borrows `chunk`, itself borrowed from `samples`, never owns it
            let sum: i32 = chunk.iter().sum();
            println!("chunk sum: {sum}");
        });
    }
}); // every scoped thread is joined here; `samples` is still valid afterward
```

**Why this way:** `thread::scope` guarantees every spawned thread finishes
before the scope returns, which is what lets threads borrow local data
directly instead of requiring the `'static` lifetime (and usually an
`Arc`) that plain `thread::spawn` demands — see the
[std docs for `thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html).

## Explanation (Embedded)

Borrowing means exactly the same thing under `#![no_std]` — a shared
reference (`&T`) grants temporary read-only access without transferring
ownership, any number of `&T` can coexist as long as no `&mut T` to the
same data is alive at the same time, and none of it has any runtime
representation. What's distinctly common in HAL-adjacent code is passing
a `&Peripheral`/`&Config`-shaped value into multiple driver calls without
ever giving up ownership of the underlying value: a configuration struct
assembled once in `main` is typically borrowed by every driver `init` call
that needs to read it, rather than cloned per call (a real, avoidable
RAM/cycle cost on a constrained device — see
[Copy vs Clone](copy-vs-clone.md)) or moved into just one of them, which
would leave the rest of the program with nothing to read. The same `&T`/
`&mut T` exclusivity rule embedded code leans on for interrupt safety (see
[The borrow checker](borrow-checker.md)) is this same mechanism at
higher stakes; ordinary shared borrowing of config values and peripheral
handles across driver code is its everyday, lower-stakes case.

## Basic usage example (Embedded)

```
struct SensorConfig { gain: u8, sample_rate_hz: u16 }

fn init_temperature(cfg: &SensorConfig) { /* ... */ }

let config = SensorConfig { gain: 4, sample_rate_hz: 100 };
init_temperature(&config); // <- borrows: init_temperature never takes ownership
println!("{}", config.gain); // config is still usable afterward
```

## Best practices & deeper information (Embedded)

### Scenario: Sharing data with multiple references

A configuration struct built once in `main` is read by two separate
sensor driver init calls, neither of which needs to own it.

```
struct SensorConfig { gain: u8, sample_rate_hz: u16 }

fn init_temperature(cfg: &SensorConfig) { /* configure the temperature ADC channel */ }
fn init_humidity(cfg: &SensorConfig) { /* configure the humidity ADC channel */ }

let config = SensorConfig { gain: 4, sample_rate_hz: 100 };
init_temperature(&config); // <- first shared borrow
init_humidity(&config);    // <- second shared borrow of the same config, both read-only
```

**Why this way:** cloning `config` for each call would duplicate it for no
reason (neither driver keeps a copy), and moving it into one call would
leave the other with nothing to read — sharing the same `&T` across both
is the natural fit, and, on a device with a few KB of RAM total, also the
cheapest one; the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers unlimited concurrent shared borrows as the direct benefit of `&T`
never granting write access.

### Scenario: Designing a public API

A function that only reads a config should accept a shared reference
rather than take ownership of it, so the caller can keep using that value
for every other init call afterward.

```
// PREFER: accept a shared reference — the caller keeps ownership of `cfg`
fn validate(cfg: &SensorConfig) -> bool {
    cfg.sample_rate_hz <= 1000
}

// AVOID: taking ownership here forces the caller to clone `cfg` or give it up
// fn validate_owned(cfg: SensorConfig) -> bool { cfg.sample_rate_hz <= 1000 }

let config = SensorConfig { gain: 4, sample_rate_hz: 100 };
if validate(&config) {
    init_temperature(&config); // <- config is still available: validate() only ever borrowed it
}
```

**Why this way:** a function that only needs to read should ask for no
more than a shared reference — taking ownership it doesn't need would
force every caller to either clone the value or give up their own access
to it, exactly the kind of unnecessary restriction the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html)
recommend against when a borrow is all an API actually requires.

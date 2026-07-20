---
title: "The borrow checker"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Java / C#", "Coming from C / C++"]
related_syntax: ["&", mut]
see_also: ["Ownership", "Borrowing (shared references)", "Mutable borrowing", "Lifetimes"]
---

## Explanation

The borrow checker is the part of the compiler that statically verifies
every ownership and borrowing rule holds for every possible execution
path, before the program ever runs: no value is used after it's moved, no
reference outlives the data it points to, and no mutable reference
coexists with any other reference to the same data.

This is what most people mean when they talk about "fighting the borrow
checker" as a newcomer — it rejects programs that would be memory-unsafe
(dangling pointers, data races, use-after-free) at compile time, with no
runtime cost and no possibility of the bug reaching production, in
exchange for sometimes requiring the code to be restructured in ways that
feel unfamiliar coming from a language without this check.

The mental model that helps most: the borrow checker isn't an arbitrary
obstacle, it's checking a real property your program needs to hold
regardless of language — other languages either enforce a version of it
at runtime (garbage collection sidesteps use-after-free by never freeing
early; a mutex enforces exclusivity at runtime with a lock) or don't
enforce it at all (raw pointers in C, where violating it is undefined
behavior). Rust's distinguishing choice is doing the check entirely at
compile time, for zero runtime cost, which is only possible because the
rules are conservative — some genuinely safe programs are rejected simply
because the checker can't prove they're safe, which is why escape hatches
like [interior mutability](interior-mutability.md), reference counting,
and (in the rare, unavoidable case) `unsafe` exist.

## Basic usage example

```
let mut v = vec![1, 2, 3];
let first = &v[0];       // <- shared borrow of v starts here
v.push(4);                // borrow checker error: can't mutate v...
println!("{first}");      // ...while `first` still borrows it
```

**Restriction:** this is a compile-time rejection, not a runtime panic —
the fix is to shorten `first`'s scope (or restructure the code) so it no
longer overlaps with the mutable use.

## Best practices & deeper information

### Scenario: Sharing data with multiple references

Reading a shared config in several places goes smoothly as long as each
borrow's scope is kept as small as the code that actually needs it,
rather than held open for the rest of a function "just in case."

```
struct Config {
    max_retries: u32,
    timeout_ms: u32,
}

fn summarize(cfg: &Config) -> String {
    format!("retries={} timeout={}", cfg.max_retries, cfg.timeout_ms)
}

let cfg = Config { max_retries: 3, timeout_ms: 500 };

{
    let view = &cfg; // <- borrow scoped tightly to this block
    println!("{}", summarize(view)); // <- last use: the borrow ends here (NLL)
} // well before any later mutation of cfg would need to happen
```

**Why this way:** letting a shared borrow end at its last use, rather
than at the end of its enclosing scope, is what keeps later
mutations of the same value from ever colliding with it — the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers this scoping behavior as the key to avoiding borrow-checker
friction, not a workaround for it.

### Scenario: Mutating through a reference

Updating one field of a struct while still needing to read another works
in a single scope because the borrow checker tracks borrows of individual
fields separately — a mutable borrow of one field can coexist with a
shared borrow of a different one.

```
struct Sensor {
    readings: Vec<f64>,
    status: String,
}

let mut sensor = Sensor { readings: Vec::new(), status: "ok".into() };

let readings = &mut sensor.readings; // <- mutable borrow of ONE field
let status = &sensor.status;         // <- simultaneous shared borrow of a DIFFERENT field
readings.push(21.5);
println!("{status}");
```

**Why this way:** the borrow checker tracks field-level (disjoint)
borrows within a function body, so structuring mutation around individual
fields rather than whole-struct `&mut self` methods avoids artificial
conflicts; the
[Rustonomicon](https://doc.rust-lang.org/nomicon/borrow-splitting.html)
covers splitting borrows across separate fields as the standard way to
keep the checker satisfied without restructuring the data itself.

## Embedded Rust Notes

**Full support.** The borrow checker runs at compile time on the host
toolchain regardless of what target you're compiling for — it applies
identically whether the output binary targets a desktop or a
Cortex-M microcontroller.

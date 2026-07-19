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
    println!("{}", summarize(view));
} // borrow ends here, well before any later mutation of cfg would need to happen
```

**Why this way:** letting a shared borrow's scope end at its last actual
use, rather than its last syntactic appearance, is what keeps later
mutations of the same value from ever colliding with it — the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers this scoping behavior as the key to avoiding borrow-checker
friction, not a workaround for it.

### Scenario: Mutating through a reference

Updating one field of a struct while still needing to read another is
best done by borrowing only the field that changes, rather than taking
`&mut self` on the whole struct for a change that only touches one part
of it.

```
struct Sensor {
    readings: Vec<f32>,
    last_error: Option<String>,
}

fn record(sensor: &mut Sensor, value: f32) {
    sensor.readings.push(value); // <- mutable borrow limited to the `readings` field
}

let mut sensor = Sensor { readings: Vec::new(), last_error: None };
record(&mut sensor, 21.5);

if let Some(err) = &sensor.last_error { // fine: no mutable borrow of `sensor` is still open
    println!("last error: {err}");
}
```

**Why this way:** structuring code so a mutation only ever borrows the
field it changes — instead of the whole struct — keeps borrows narrow
enough that unrelated reads of the same value don't conflict with them;
the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers disjoint borrows of separate fields as one of the standard ways to
keep the checker satisfied without restructuring the data itself.

## Embedded Rust Notes

**Full support.** The borrow checker runs at compile time on the host
toolchain regardless of what target you're compiling for — it applies
identically whether the output binary targets a desktop or a
Cortex-M microcontroller.

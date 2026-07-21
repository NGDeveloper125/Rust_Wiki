---
title: "Zero-cost abstractions"
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Systems / Low-Level Programming", "Unique to Rust", "Coming from C / C++"]
related_syntax: []
see_also: ["Static dispatch & monomorphization", "Generics", "Iterator adaptors", "Const generics", "Memory safety without a garbage collector"]
---

## Explanation

A zero-cost abstraction is a language feature that lets you write code at a
higher level — generic functions, iterator chains, trait-based
polymorphism — without paying any runtime price for having written it that
way, compared to the equivalent code written out by hand at the lower
level. The phrase predates Rust: it's Bjarne Stroustrup's founding
principle for C++, often summarized as "what you don't use, you don't pay
for, and what you do use, you couldn't hand-code any better." Rust adopted
the same constraint but applied it far more broadly and far more
deliberately — the abstraction facilities added to the language (generics,
traits, iterators, closures, `async`/`await`) were designed against this
bar from the outset, rather than retrofitted onto a systems language that
already had a runtime cost model of its own.

The concrete mechanism is usually [monomorphization](../traits-polymorphism/static-dispatch-monomorphization.md):
generic code is compiled once per concrete type it's actually called with,
so a call to a [generic](../types-data-modeling/generics.md) function
produces machine code indistinguishable from a hand-specialized version
for that one type — no boxed values, no runtime type dispatch, unless you
explicitly ask for it via `dyn Trait`. The same idea shows up in
[iterator adaptors](../iterators/iterator-adaptors.md): a chain of `map`,
`filter`, and `fold` calls describes a pipeline, but the compiler typically
inlines the whole chain into a single tight loop with no intermediate
collection allocated between stages — the readable, composable version and
the hand-rolled `for` loop compile down to the same instructions.
[Const generics](../types-data-modeling/const-generics.md) extend the same
promise to values baked into a type, letting a fixed-size buffer's capacity
live in the type system with the same zero runtime overhead as writing a
size-specific struct by hand. Ownership and borrowing belong on this list
too, in a subtler way: the entire discipline of single ownership and
borrow-checked references is erased once the compiler is satisfied — it
costs zero bytes and zero instructions at runtime, which is precisely what
lets Rust guarantee [memory safety without a garbage collector](memory-safety-without-a-garbage-collector.md)
instead of paying for that safety with a tracing runtime.

It's worth being honest about what "zero-cost" does *not* promise. It's a
claim about runtime cost relative to the hand-written equivalent — not
about compile time, binary size, or how easy the abstraction is to write.
Monomorphization means every concrete instantiation of a generic function
is a separate copy of machine code; lean on generics heavily across a large
codebase and both compile times and binary size grow in ways a single
non-generic function never would. Not every abstraction is free by
default, either: choosing `Box<dyn Trait>` over a generic parameter
introduces one real, measurable vtable indirection per call — but that
cost is explicit and opt-in, chosen the moment you write `dyn`, rather
than a hidden tax on code that never asked for dynamic dispatch. Even
something as basic as slice indexing carries a bounds check that a raw,
unchecked C array wouldn't pay for; the compiler frequently proves the
check redundant and removes it, but "frequently" isn't "always," and the
honest version of the promise is "you're never charged for a cost you
didn't ask for, and the default, idiomatic path is the cheap one" rather
than "literally nothing here ever costs anything."

This principle is also why Rust reads as trustworthy to programmers coming
from C or C++: reaching for a `Vec<T>` instead of hand-managed heap memory,
or a trait bound instead of a function pointer table, isn't a tradeoff
against performance the way it would be when reaching for an abstraction
in most higher-level languages. The abstraction is the idiomatic choice
specifically because it doesn't cost anything beyond the lower-level
alternative — which is also why [fearless concurrency](fearless-concurrency.md)
and compile-time-checked marker traits like `Send`/`Sync` fit the same
mold: the safety guarantee is entirely a compile-time artifact, with no
runtime representation left behind in the compiled program.

## Basic usage example

```
let readings = [21.4, 19.8, 23.1, 18.6];

let above_freezing: f64 = readings
    .iter()
    .copied()
    .filter(|&c| c > 0.0) // <- adaptor: describes a transformation, allocates nothing
    .sum();                // <- consumer: pulls values through the chain one at a time
```

Compiled, this is instruction-for-instruction the same as a hand-written
`for` loop that checks each reading and accumulates a running total — the
named, composable chain costs nothing beyond what that loop would have
cost anyway.

## Best practices & deeper information

### Scenario: Writing generic code

A unit-conversion helper used with both `f32` sensor readings and `f64`
configuration values should exist once in source, while still compiling to
code as tight as two separately hand-written functions.

```
fn to_fahrenheit<T: Into<f64>>(celsius: T) -> f64 { // <- generic: monomorphized per concrete type used
    celsius.into() * 9.0 / 5.0 + 32.0
}

let sensor_reading: f32 = 21.5;
let calibration_target: f64 = 20.0;

to_fahrenheit(sensor_reading);      // compiler emits a to_fahrenheit::<f32> copy
to_fahrenheit(calibration_target);  // ...and a separate to_fahrenheit::<f64> copy, both resolved at compile time
```

**Why this way:** writing the conversion once as a generic function avoids
duplicating it per numeric type, and monomorphization means neither
instantiation runs any slower than a version hand-written specifically for
`f32` or `f64` — the
[Rust Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
covers this compile-time specialization as the reason generic code carries
no runtime penalty.

### Scenario: Numeric computation

Averaging only the in-range readings from a sensor log reads clearly as a
filter-then-fold chain, and compiles to the same single pass a hand-written
loop would take — no intermediate `Vec` is allocated for the filtered
subset.

```
struct Reading { celsius: f64, valid: bool }

let readings = [
    Reading { celsius: 21.4, valid: true },
    Reading { celsius: -999.0, valid: false }, // sensor fault code, not a real temperature
    Reading { celsius: 19.8, valid: true },
];

let (sum, count) = readings
    .iter()
    .filter(|r| r.valid)  // <- adaptor: skips faulty readings lazily, no allocation
    .map(|r| r.celsius)   // <- adaptor: projects to just the value needed
    .fold((0.0, 0), |(sum, count), c| (sum + c, count + 1)); // <- consumer: single pass over the data

let average = sum / count as f64;
```

**Why this way:** chaining `filter`/`map`/`fold` instead of collecting into
an intermediate `Vec` avoids an allocation the hand-rolled equivalent
wouldn't need either; the
[Rust Book's loops-vs-iterators comparison](https://doc.rust-lang.org/book/ch13-04-performance.html)
measures this directly and finds iterator chains compiling to code as fast
as the equivalent manual loop.

## Embedded Rust Notes

**Full support** — and arguably the principle that matters most on
constrained hardware. Flash and RAM budgets leave no room for a hidden
runtime cost, so the fact that generics, iterator chains, and trait
dispatch (when chosen as static dispatch) compile away to the same code a
hand-written C-style loop would produce is precisely why embedded Rust can
offer high-level ergonomics — the `embedded-hal` ecosystem chief among
them — without the overhead that would rule out a garbage-collected or
heavily runtime-typed language on the same device.

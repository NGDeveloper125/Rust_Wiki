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

## Explanation (Embedded)

Of every principle on this site, this is arguably the one that matters
most to whether embedded Rust is usable at all. A microcontroller's flash
and RAM budget is measured in kilobytes, not gigabytes, and its clock
budget for a control loop or interrupt handler is measured in a handful of
microseconds — there is no room to pay even a small, "acceptable on a
server" tax for writing readable code. If iterator chains, generics, and
`Option`/`Result` carried any real overhead compared to a hand-rolled C
loop and a raw integer flag, embedded developers simply couldn't use them,
the same way they mostly can't reach for a garbage-collected language's
convenience features on this hardware. Zero-cost abstractions are what
make that tradeoff not exist in the first place: a `for reading in
sensor_readings.iter().filter(...)` chain compiles to the identical
machine code a hand-written indexing loop would produce, so choosing the
readable version costs nothing measurable in flash size or cycle count.

The same story extends past iterators. `Option<T>` and `Result<T, E>` are
ordinary enums, and the compiler's niche-optimization pass frequently
stores their discriminant in an already-unused bit pattern of `T` rather
than adding a separate tag byte — `Option<&T>` and `Option<NonZeroU8>` are
each exactly the size of the value they wrap, with `None` represented by
the one bit pattern (null, zero) the wrapped type can't otherwise take.
That matters concretely on a device where a handful of extra bytes per
struct, multiplied across every peripheral-configuration value in a
firmware image, is a real fraction of the RAM budget. Generics compiled
via monomorphization are what let the `embedded-hal` ecosystem exist at
all: a HAL trait like `OutputPin` can be implemented identically for a
dozen different chip families, and code written generically against
`impl OutputPin` compiles, per concrete pin type, down to the exact
sequence of register writes a hand-specialized function for that one chip
would have used — no vtable, no indirect call, unless a driver explicitly
opts into `dyn OutputPin` for real runtime polymorphism (e.g. a bootloader
that genuinely doesn't know which board variant it's running on until
runtime).

The honest caveat from the classic explanation applies with extra force
here, too: monomorphization means every concrete instantiation of a
generic HAL function is a separate copy of machine code, and on a chip
with 32 or 64 KB of flash total, leaning on generics across many
peripheral types can grow the binary enough to matter in a way it simply
wouldn't on a hosted target with gigabytes of storage. Choosing `dyn Trait`
over a generic parameter is the same explicit, opt-in tradeoff described
above — one real vtable indirection per call, worth it only when the
flash savings from a single non-duplicated function body outweigh the
cycle cost of the indirect call, which on some peripheral-heavy firmware
it genuinely does.

## Basic usage example (Embedded)

```
let readings: [i16; 4] = [212, -999, 198, 205]; // raw ADC counts; -999 marks a faulty sample

let total: i32 = readings
    .iter()
    .copied()
    .filter(|&r| r != -999) // <- adaptor: drops the fault sentinel, allocates nothing
    .map(i32::from)          // <- adaptor: widens before summing, still zero-cost
    .sum();                  // <- consumer: single pass, compiles to the same loop a hand-rolled `for` would
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

Averaging a fixed array of ADC samples, skipping any that read as a fault
sentinel, needs to run inside a tight interrupt handler with no
allocation and no wasted cycles — an iterator chain over the raw array
gets there without hand-writing the loop.

```
const FAULT_SENTINEL: i16 = -999;

fn average_valid_readings(samples: &[i16; 8]) -> Option<i32> { // <- fixed-size array: no heap, no Vec, no allocation
    let (sum, count) = samples
        .iter()
        .copied()
        .filter(|&s| s != FAULT_SENTINEL) // <- adaptor: lazily skips faults, no intermediate buffer allocated
        .fold((0i32, 0i32), |(sum, count), s| (sum + s as i32, count + 1)); // <- consumer: one pass over the array

    if count == 0 { None } else { Some(sum / count) }
}
```

**Why this way:** the compiled loop here is indistinguishable from a
hand-written `for i in 0..8 { ... }` version — the
[Rust Book's loops-vs-iterators comparison](https://doc.rust-lang.org/book/ch13-04-performance.html)
measures exactly this equivalence on hosted targets, and the same
monomorphized-adaptor compilation applies under `#![no_std]`, which is why
this idiom shows up throughout `embedded-hal`-based sensor-processing code
without a size or speed tax.

### Scenario: Writing generic code

A single "blink" routine should work against any GPIO pin type from any
supported chip family, compiling down to the same direct register writes
a version hand-written for one specific chip would use.

```
use embedded_hal::digital::OutputPin;

fn blink<P: OutputPin>(pin: &mut P, delay_ms: impl Fn(u32)) { // <- generic: monomorphized per concrete pin type
    pin.set_high().ok();
    delay_ms(200);
    pin.set_low().ok();
    delay_ms(200);
}
```

**Why this way:** calling `blink` with an `stm32`-family pin and, in a
different build, with an `nrf`-family pin produces two separate,
independently-optimized copies of the function — each compiling to that
chip's own direct register-write instructions, with no vtable and no
runtime dispatch — which is exactly why the `embedded-hal` trait
ecosystem can offer one portable API across chip families without
imposing a per-call indirection cost on any of them.

### Scenario: Bit manipulation and flags

A sensor's calibration offset register is either "not yet calibrated" or
holds a nonzero calibration value — modeling that as `Option<NonZeroU16>`
instead of a separate `bool` flag plus a `u16` keeps the struct at exactly
2 bytes, with no discriminant byte spent on the tag.

```
use core::num::NonZeroU16;

struct Calibration {
    offset: Option<NonZeroU16>, // <- niche-optimized: None reuses the all-zero bit pattern NonZeroU16 can't hold
}

fn apply_calibration(raw_reading: u16, cal: &Calibration) -> u16 {
    match cal.offset {
        Some(offset) => raw_reading.wrapping_add(offset.get()),
        None => raw_reading, // not yet calibrated: pass the raw reading through unchanged
    }
}
```

**Why this way:** `size_of::<Option<NonZeroU16>>()` is 2, identical to
`size_of::<NonZeroU16>()` and to `size_of::<u16>()`, because the compiler
stores `None` in the all-zero pattern `NonZeroU16` is guaranteed never to
hold — a `bool` flag alongside a plain `u16` would cost at least one extra
byte (likely more once alignment padding is counted) per calibration
value, real savings once a firmware image tracks calibration state for
several sensors on a RAM-constrained target.

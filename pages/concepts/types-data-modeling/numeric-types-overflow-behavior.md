---
title: "Numeric types & overflow behavior"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Numeric Safety"]
related_syntax: [integer-suffixes, float-suffixes]
see_also: []
---

## Explanation

Rust's integer types are explicit about width and signedness —
`u8`/`i8` through `u128`/`i128`, plus pointer-sized `usize`/`isize` — with
no single generic "number" type and no implicit widening between
different integer types (adding a `u8` and a `u32` directly is a compile
error; an explicit `as` cast or `.into()` conversion is required).

Overflow behavior is deliberately different between build profiles: in a
debug build, an operation that overflows its type's range (`255u8 + 1`)
panics immediately, surfacing the bug during development. In a release
build, the same operation instead **wraps** silently (`255u8 + 1 == 0`)
for performance reasons — checking every arithmetic operation at runtime
in optimized code would cost real, measurable overhead. Where either
behavior isn't good enough — you need guaranteed, defined behavior
regardless of build profile — explicit methods make the choice visible in
the code itself: `checked_add` (returns `None` on overflow),
`wrapping_add` (always wraps), `saturating_add` (clamps to the type's
max/min), and `overflowing_add` (returns the wrapped value plus a bool
flag).

This design trades a small amount of implicit safety (debug-only
overflow panics) for explicitness everywhere it actually matters, rather
than picking one runtime behavior and making every caller pay for it
unconditionally.

## Basic usage example

```
let a: u8 = 250;
let b: u8 = 10;

let sum = a.checked_add(b); // <- None: 260 doesn't fit in a u8, caught explicitly
println!("{sum:?}");
```

**Restriction:** writing plain `a + b` here instead hides the same
problem — it panics in a debug build but silently wraps to `4` in a
release build, so the two profiles behave differently unless you use an
explicit `checked_`/`wrapping_`/`saturating_` method.

## Best practices & deeper information

### Scenario: Numeric computation

Widening an accumulator before summing narrow values sidesteps overflow
entirely, which is cheaper than reaching for checked arithmetic for what
is otherwise ordinary, bounded addition.

```
let sensor_readings: [u16; 4] = [1200, 980, 1500, 1100];

let total: u32 = sensor_readings.iter().map(|&r| r as u32).sum(); // <- widen before summing: u16 could overflow
let average = total / sensor_readings.len() as u32;

println!("average reading: {average}");
```

**Why this way:** summing narrow integers directly risks overflowing the
narrow type the moment enough values accumulate; widening to a type with
real headroom avoids the risk up front instead of needing
`checked_add` at every step — the kind of unchecked arithmetic Clippy's
[`arithmetic_side_effects`](https://rust-lang.github.io/rust-clippy/master/#arithmetic_side_effects)
lint flags when it can't prove overflow is impossible.

### Scenario: Validating input

Once a value originates outside the program — a request body, a config
file, anything the caller could get wrong — plain arithmetic on it trusts
an assumption untrusted input is specifically there to violate.
`checked_*` turns that into a `Result` instead of a silent wraparound.

```
fn apply_discount(price_cents: u32, discount_cents: u32) -> Result<u32, &'static str> {
    price_cents
        .checked_sub(discount_cents) // <- untrusted input: a discount larger than the price must not silently wrap
        .ok_or("discount exceeds price")
}

apply_discount(500, 100); // Ok(400)
apply_discount(500, 900); // Err("discount exceeds price") instead of wrapping to a huge u32
```

**Why this way:** a release build silently wraps on overflow rather than
panicking, so `price_cents - discount_cents` on untrusted input would
compile and often "work" in testing, then wrap to a near-`u32::MAX`
value in production the first time a caller sends bad data —
`checked_sub` makes that failure explicit and impossible to ignore.

## Embedded Rust Notes

**Full support.** All defined in `core`, no `std` dependency. Overflow
behavior is worth extra attention in embedded code: register widths
often dictate the natural integer type (`u16` for a 16-bit ADC reading,
for example), and the debug-panics/release-wraps split still applies —
safety-critical embedded code frequently uses explicit
`checked_`/`saturating_` arithmetic deliberately, rather than relying on
build-profile-dependent default behavior.

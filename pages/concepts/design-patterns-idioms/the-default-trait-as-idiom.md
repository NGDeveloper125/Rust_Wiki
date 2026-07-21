---
title: "The Default trait as idiom"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Builders & Object Construction"]
related_syntax: [derive]
see_also: ["Constructor functions (new() convention)", "Derivable traits (Debug, Clone, PartialEq, …)", "Structs"]
---

## Explanation

`Default` is a trait with one method, `default() -> Self`, that produces
a reasonable "starting" or "empty" value for a type — zero for numbers,
an empty string or empty collection, `None` for an `Option`. Its
usefulness as an idiom goes well past being a convenient zero-argument
constructor: because it's a trait rather than a naming convention, generic
code can require it (`fn make<T: Default>() -> T`) and produce a fresh
value of *some* type it knows nothing else about, which a bare `new()`
convention can never offer — `new` isn't a trait, so nothing generic can
call it.

For an ordinary struct made entirely of fields that themselves implement
`Default`, `#[derive(Default)]` generates the impl mechanically — every
field gets its own default, with no need to write the method by hand.
This turns "give me a blank one of these" into a one-line attribute for
the overwhelming majority of plain data structs, the same way
`#[derive(Debug, Clone, PartialEq)]` mechanizes those traits (see
[Derivable traits](../traits-polymorphism/derivable-traits.md)).

The idiom's most visible payoff is struct update syntax: `SomeConfig {
timeout_ms: 500, ..Default::default() }` sets only the fields a caller
actually cares about and fills every other field from its type's default.
This is the idiomatic shape for a config-like struct with many optional
settings — instead of a builder with a setter per field, or a constructor
with a dozen parameters most callers would pass as placeholders, the
caller writes only the overrides that matter and the rest disappears into
`..Default::default()`.

`Default` and [`new()`](constructor-functions-new.md) are closely related
idioms rather than competitors: when a type's "obvious" starting value
*is* its default, the usual advice is to implement both and have one call
the other (`new` delegating to `Default::default()`, or the reverse) so
they can never silently drift apart into two different "empty" values for
the same type.

## Basic usage example

```
#[derive(Default)] // <- generates default(): every field gets its own type's default
struct RetryPolicy {
    max_attempts: u32, // defaults to 0
    backoff_ms: u64,   // defaults to 0
}

let policy = RetryPolicy::default();
println!("{}", policy.max_attempts);
```

## Best practices & deeper information

### Scenario: Creating a new object

A server config has one field most callers care about and several they
almost never touch — struct update syntax against `Default::default()`
lets a caller set only what matters.

```
#[derive(Debug)]
struct ServerConfig {
    port: u16,
    max_connections: u32,
    timeout_ms: u64,
    tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self { // <- manual impl: the "sensible" defaults aren't all zero
        ServerConfig { port: 8080, max_connections: 100, timeout_ms: 30_000, tls: false }
    }
}

let config = ServerConfig {
    port: 9090, // <- only the override the caller actually needs
    ..Default::default() // <- every other field comes from ServerConfig::default()
};
println!("{config:?}");
```

**Why this way:** struct update syntax against `Default::default()` scales
to configs with many optional fields without a builder or a huge
constructor signature, which the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/default.html)
book documents as the idiomatic way to handle "mostly-optional" struct
fields.

### Scenario: Writing generic code

A function that needs to produce a placeholder value of a type it only
knows through a generic parameter can require `Default` and call it,
something a `new()` convention alone could never offer since it isn't a
trait a generic bound can name.

```
#[derive(Default)]
struct Metrics {
    requests: u64,
    errors: u64,
}

fn reset<T: Default>(slot: &mut T) {
    *slot = T::default(); // <- works for any T that implements Default, not just Metrics
}

let mut metrics = Metrics { requests: 42, errors: 3 };
reset(&mut metrics);
println!("{} {}", metrics.requests, metrics.errors);
```

**Why this way:** bounding a generic function on `Default` is the only
way to say "give me a fresh value of whatever `T` turns out to be,"
since `new()` is just a name and cannot be required by a trait bound —
the
[std docs for `Default`](https://doc.rust-lang.org/std/default/trait.Default.html)
call this generic-code usage out as a primary reason the trait exists
alongside plain constructors.

## Explanation (Embedded)

`Default` is defined in `core`, not `std` or `alloc`, so both the trait
and `#[derive(Default)]` work identically under `#![no_std]` — there's no
allocator dependency anywhere in this idiom. It earns a particularly
natural place in embedded code because so many peripheral config structs
have one obvious "most common configuration": a UART's most common setup
really is 9600 baud, no parity, one stop bit; a GPIO pin's most common
mode really is a floating input. Implementing `Default` for that struct
lets firmware write `UartConfig { baud_rate: 115_200, ..Default::default() }`
when it only cares about overriding the baud rate, instead of naming
every field, and lets generic HAL-adjacent code request "a default
config of whatever peripheral type I'm generic over" through a trait
bound the same way hosted Rust does.

## Basic usage example (Embedded)

```
#[derive(Default)] // <- generates default(): every field gets its own type's default
struct GpioConfig {
    output: bool,    // defaults to false: floating input, the safest reset state
    pull_up: bool,   // defaults to false
}

let config = GpioConfig::default();
```

## Best practices & deeper information (Embedded)

### Scenario: Creating a new object

A UART peripheral has one field firmware almost always cares about
(baud rate) and several that are fine left at their most common setting
— struct update syntax against `Default::default()` lets a caller set
only what matters, without a full builder.

```
#[derive(Debug)]
struct UartConfig {
    baud_rate: u32,
    parity: bool,
    stop_bits: u8,
}

impl Default for UartConfig {
    fn default() -> Self { // <- manual impl: the "sensible" defaults aren't all zero
        UartConfig { baud_rate: 9_600, parity: false, stop_bits: 1 }
    }
}

let config = UartConfig {
    baud_rate: 115_200, // <- only the override this firmware actually needs
    ..Default::default() // <- parity and stop_bits come from UartConfig::default()
};
```

**Why this way:** struct update syntax against `Default::default()`
scales to a peripheral config with several optional settings without a
builder or a long constructor signature, which the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/default.html)
book documents as the idiomatic way to handle "mostly-optional" struct
fields — and a UART's 8N1-at-9600 baud setup is exactly the kind of
"obvious common case" this idiom is built for.

### Scenario: Writing generic code

A driver-reset routine needs to put a peripheral's in-memory state
struct back to a known-safe starting point after detecting a fault,
without knowing at compile time which specific peripheral's state struct
it's holding.

```
#[derive(Default)]
struct AdcState {
    last_reading: u16,
    overrun_count: u32,
}

fn reset<T: Default>(state: &mut T) {
    *state = T::default(); // <- works for any peripheral state type that implements Default
}

let mut adc_state = AdcState { last_reading: 512, overrun_count: 3 };
reset(&mut adc_state);
```

**Why this way:** bounding a generic fault-recovery routine on `Default`
is the only way to say "put whatever state type `T` turns out to be back
to its known-safe starting point," since a bare `new()` convention isn't
a trait a generic bound can require — the
[std docs for `Default`](https://doc.rust-lang.org/std/default/trait.Default.html)
call out generic code as a primary reason the trait exists, and it applies
unchanged to zeroing register-mirroring structs or protocol frames back
to a known-safe state in `#![no_std]` firmware.

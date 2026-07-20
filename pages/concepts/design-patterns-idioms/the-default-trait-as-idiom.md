---
title: "The Default trait as idiom"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Builders & Object Construction"]
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

## Embedded Rust Notes

**Full support.** `Default` is defined in `core`, and `#[derive(Default)]`
works identically under `#![no_std]`. It's especially convenient for
zeroing out register-mirroring structs or protocol frames to a known-safe
starting state without hand-writing every field.

---
title: "Enums (algebraic data types)"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Functional Programming", "Designing Robust Data Models", "State Machines", "Recursive Data Structures", "Coming from Java / C#", "Coming from Haskell / functional languages"]
related_syntax: [enum]
see_also: ["match expressions", "Exhaustiveness checking", "\"Make invalid states unrepresentable\""]
---

## Explanation

An enum defines a type as one of several distinct variants, each of which
can optionally carry its own data — for example, a `Shape` enum might
have a tuple-style `Circle(f64)` variant, a `Rectangle(f64, f64)` variant,
and a struct-style `Triangle { base: f64, height: f64 }` variant, all as
alternatives of the same type.

This is what's meant by "algebraic data type" — a *sum* type, where a
value is exactly one of several alternatives (as opposed to a struct, a
*product* type, where a value is all of its fields at once). This is
categorically more powerful than the "enum" in languages like C, Java, or
C# (before their more recent additions), where an enum is just a named
set of integer-like constants — a Rust enum variant can carry arbitrary,
variant-specific data, which is what makes `Option<T>` and `Result<T, E>`
possible as ordinary enums rather than special-cased language features.

Combined with [`match`](../pattern-matching/match-expressions.md), enums
are the primary tool for making illegal states genuinely unrepresentable:
a value can only ever be one of the variants you defined, each variant
can only carry exactly the data appropriate to that case, and the
compiler forces every `match` to handle every variant (see
[Exhaustiveness checking](../pattern-matching/exhaustiveness-checking.md)) —
so adding a new variant later surfaces every place in the codebase that
needs updating, as a compile error, rather than a silent gap in behavior.

## Basic usage example

```
enum Status {
    Active,
    Paused(u32),                // <- this variant carries its own data
    Stopped { reason: String },
}

let s = Status::Paused(30);
match s {
    Status::Active => println!("running"),
    Status::Paused(secs) => println!("paused for {secs}s"), // <- data is extracted per-variant
    Status::Stopped { reason } => println!("stopped: {reason}"),
}
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

An exhaustive `match` on an enum is the idiomatic way to branch on which
variant a value currently is, extracting each variant's data by name
instead of reaching into it afterward.

```
enum SensorReading {
    Valid(f64),
    OutOfRange { value: f64, limit: f64 },
    SensorOffline,
}

fn describe(reading: &SensorReading) -> String {
    match reading { // <- exhaustive: every variant must be handled, or this doesn't compile
        SensorReading::Valid(v) => format!("{v:.1}"),
        SensorReading::OutOfRange { value, limit } => format!("{value:.1} exceeds limit {limit:.1}"),
        SensorReading::SensorOffline => "offline".to_string(),
    }
}
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html) covers
`match`'s exhaustiveness as central to enums' value — adding a new
`SensorReading` variant later turns every `match` that needs updating
into a compile error, instead of a silently-missing case.

### Scenario: Handling and propagating errors

A custom error enum lets each failure mode carry exactly the data that
explains it, so callers can react to *which* case happened instead of
parsing a generic error message.

```
enum ConfigError {
    MissingField(String),          // <- each variant carries only the data relevant to that failure
    InvalidValue { field: String, reason: String },
    Io(std::io::Error),
}

fn load_port(raw: Option<&str>) -> Result<u16, ConfigError> {
    let raw = raw.ok_or_else(|| ConfigError::MissingField("port".into()))?;
    raw.parse().map_err(|_| ConfigError::InvalidValue {
        field: "port".into(),
        reason: "not a valid u16".into(),
    })
}
```

**Why this way:** structuring errors as a data-carrying enum rather than
a string is one of [Effective Rust](https://effective-rust.com/)'s core
error-handling recommendations — it lets a caller `match` on the failure
and handle each case differently, which a formatted error message alone
can't support.

### Scenario: Serializing and deserializing

`serde` can tag which variant a JSON object represents in more than one
shape; picking the internally-tagged form keeps the output flat instead
of nesting each variant's data under its own extra key.

```
// [dependencies] serde = { version = "1", features = ["derive"] }, serde_json = "1"
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")] // <- variant name is stored under a "type" key instead of nesting it
enum Event {
    Connected { addr: String },
    Disconnected { reason: String },
}

let json = serde_json::to_string(&Event::Connected { addr: "10.0.0.5".into() }).unwrap();
// {"type":"Connected","addr":"10.0.0.5"}
```

**Why this way:** serde's default "externally tagged" representation
round-trips fine but nests each variant's fields under their own key,
which is awkward for non-Rust consumers;
[serde's enum representations guide](https://serde.rs/enum-representations.html)
covers `#[serde(tag = "...")]` as the option to reach for when a flatter
JSON shape is what the wire format actually needs.

## Embedded Rust Notes

**Full support.** Enums are core-language and allocator-free (their size
is roughly the largest variant plus, when needed, a discriminant —
though niche optimization often folds the tag into a variant's unused bit
patterns, so e.g. `Option<&T>` stays pointer-sized) — ideal for
representing peripheral states, register field values, or protocol
message types with zero runtime allocation.

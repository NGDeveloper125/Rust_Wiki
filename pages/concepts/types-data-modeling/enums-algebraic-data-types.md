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

## Explanation (Embedded)

Enums are core-language and allocator-free — their size is roughly the
largest variant plus, when needed, a discriminant tag (niche optimization
often folds that tag into a variant's unused bit patterns instead, so
e.g. `Option<&T>` stays pointer-sized) — which makes them a natural fit
for representing a peripheral's discrete states, fault codes, or protocol
message types with zero runtime allocation. See
["Make invalid states unrepresentable"](../philosophy-principles/make-invalid-states-unrepresentable.md)'s
embedded section for the full case of *why* this matters specifically for
hardware — modeling a register field as an enum rather than a bare
integer means only the datasheet's documented bit patterns can exist as
values of that type at all; this page's job is the mechanics of the enum
itself rather than repeating that argument.

The mechanic worth covering here is `#[repr(u8)]` (or `u16`/`u32`): by
default, Rust picks the enum's discriminant values and backing integer
width itself, which is exactly right when nothing outside the program
ever needs to see those numbers. The moment a discriminant has to match a
*hardware-defined* encoding — a status register's mode-select bits, a
protocol's message-type byte — `#[repr(u8)]` plus explicit `= value`
assignments on each variant pin the enum's numeric representation to
match the datasheet exactly, so `SomeEnum::Variant as u8` produces the
precise byte the hardware expects, and `TryFrom<u8>` (as the
cross-referenced page shows) converts a raw register read back into the
enum, rejecting whatever bit patterns the datasheet leaves undefined.

## Basic usage example (Embedded)

```
#[repr(u8)] // <- pins the discriminant width and layout to match a hardware encoding
enum PowerMode {
    Active = 0x01,
    Idle = 0x02,
    Sleep = 0x04,
    DeepSleep = 0x08,
}

fn write_power_mode(mode: PowerMode) -> u8 {
    mode as u8 // <- exact byte the peripheral's power-control register expects
}

let reg_value = write_power_mode(PowerMode::Sleep); // 0x04
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A peripheral's fault-status register packs several distinct, mutually
exclusive fault codes into one byte; modeling them as a `#[repr(u8)]`
enum with datasheet-matching discriminants means the numeric encoding is
fixed exactly once, at the type definition, rather than as magic numbers
scattered through every place the register is read.

```
#[repr(u8)]
#[derive(Debug, PartialEq)]
enum FaultCode {
    None = 0x00,
    Overcurrent = 0x01,
    Overtemperature = 0x02,
    UnderVoltage = 0x04,
}

impl TryFrom<u8> for FaultCode {
    type Error = u8;
    fn try_from(raw: u8) -> Result<Self, u8> {
        match raw {
            0x00 => Ok(FaultCode::None),
            0x01 => Ok(FaultCode::Overcurrent),
            0x02 => Ok(FaultCode::Overtemperature),
            0x04 => Ok(FaultCode::UnderVoltage),
            other => Err(other), // <- bit pattern the datasheet doesn't define
        }
    }
}

fn read_fault_register(raw: u8) -> Result<FaultCode, &'static str> {
    FaultCode::try_from(raw).map_err(|_| "undefined fault code")
}
```

**Why this way:** fixing the discriminants with `#[repr(u8)]` means
`FaultCode::Overcurrent as u8` is guaranteed to be `0x01` — the exact
value the datasheet documents — so the enum can be written to and read
from the real register directly, instead of maintaining a separate
mapping table between Rust-side names and hardware-side byte values.

### Scenario: Branching on data (pattern matching)

A control loop driving a peripheral through a small number of discrete
states — idle, sampling, transmitting, faulted — should dispatch on an
exhaustive `match` over an enum, so adding a new state later is a compile
error everywhere the loop needs updating, not a silently unhandled case
in a real-time loop.

```
enum AdcState {
    Idle,
    Sampling { channel: u8 },
    Transmitting,
    Faulted(FaultCode),
}

fn step(state: AdcState) -> AdcState {
    match state { // <- exhaustive: a new AdcState variant fails to compile here until handled
        AdcState::Idle => AdcState::Sampling { channel: 0 },
        AdcState::Sampling { channel } if channel < 3 => AdcState::Sampling { channel: channel + 1 },
        AdcState::Sampling { .. } => AdcState::Transmitting,
        AdcState::Transmitting => AdcState::Idle,
        AdcState::Faulted(code) => AdcState::Faulted(code), // <- stays faulted until explicitly reset
    }
}
```

**Why this way:** a control loop that runs for the life of the device
can't afford a state transition table with a silently-missing branch —
the exhaustiveness check turns "we added a new state and forgot to handle
it somewhere" into a build failure instead of a fault that only shows up
once the device is already deployed.

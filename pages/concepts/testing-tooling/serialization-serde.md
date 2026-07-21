---
title: "Serialization (the serde ecosystem)"
area: "Testing & Tooling"
embedded_support: partial
groups: ["Testing & Tooling", "Serialization"]
related_syntax: ["#[derive(...)]"]
see_also: ["Derivable traits", "Structs", "Doc tests"]
---

## Explanation

Serialization is the process of turning a Rust value into some external
representation — JSON, TOML, a compact binary blob — and deserialization
is the reverse: rebuilding a Rust value from that representation. The
`serde` crate is the de facto standard framework for this in Rust, and
its central idea is a deliberate separation of concerns: `serde` itself
defines the generic `Serialize`/`Deserialize` traits and the machinery
for walking a type's structure, while separate crates such as
`serde_json` or `toml` implement an actual *data format* on top of that
machinery. A type implements `Serialize`/`Deserialize` exactly once and
then works with every format crate that speaks serde's traits, instead of
needing bespoke conversion code per format.

In practice, nobody writes a `Serialize`/`Deserialize` implementation by
hand for an ordinary struct or enum — `#[derive(Serialize, Deserialize)]`
generates it, using the same derive-macro mechanism described on the
[Derivable traits](../traits-polymorphism/derivable-traits.md) page,
just extended by a third-party procedural macro rather than one of the
compiler's built-in derives. That single attribute is what lets an
ordinary domain type — an order, a sensor reading, a configuration
struct — cross a serialization boundary with no hand-written glue code at
all.

`serde` field attributes give the derived implementation a small,
targeted amount of control without abandoning the derive: `#[serde(rename
= "...")]` changes the name used on the wire without renaming the Rust
field, and `#[serde(default)]` lets a field be absent from the input and
fall back to `Default::default()` instead of failing to deserialize.
Both exist for the same underlying reason — the Rust-side name and shape
of a type is allowed to evolve independently of the wire format, as long
as these attributes bridge the difference explicitly.

Serialization shows up constantly alongside other Testing & Tooling
concerns: config loaders round-trip through TOML, test fixtures are
often just deserialized JSON, and a struct's derived `Serialize` output
is frequently asserted against in a test the same way any other value
would be.

## Basic usage example

```
// [dependencies] serde = { version = "1", features = ["derive"] }, serde_json = "1"
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)] // <- generates JSON (de)serialization for this struct
struct SensorReading {
    id: u32,
    celsius: f64,
}

let reading = SensorReading { id: 7, celsius: 21.5 };
let json = serde_json::to_string(&reading).unwrap();
let parsed: SensorReading = serde_json::from_str(&json).unwrap(); // <- rebuilt from the same derive
```

## Best practices & deeper information

### Scenario: Serializing and deserializing

A configuration loader is the classic serde use case: derive
`Deserialize` on a config struct, parse a TOML file straight into it, and
lean on `#[serde(default)]` so older config files missing a newer field
still load instead of failing outright.

```
// [dependencies] serde = { version = "1", features = ["derive"] }, toml = "1"
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    #[serde(default = "default_timeout_secs")] // <- absent in old config files: falls back instead of failing
    timeout_secs: u32,
}

fn default_timeout_secs() -> u32 {
    30
}

fn load_config(raw: &str) -> Result<ServerConfig, toml::de::Error> {
    toml::from_str(raw) // <- one call turns TOML text into a typed, validated struct
}
```

**Why this way:** deriving straight onto the config struct keeps the
Rust type and the file format in sync automatically, and `#[serde(default
= "...")]` lets the schema grow without breaking every config file
written before the new field existed — both are recommended directly by
[serde's own guide](https://serde.rs/attr-default.html) for handling
config evolution gracefully.

### Scenario: Designing a public API

Renaming a struct field in Rust — to fix a naming mistake or match a
project's conventions — would normally also change the wire format and
break every existing serialized payload; `#[serde(rename = "...")]`
lets the two evolve independently.

```
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct OrderRecord {
    id: u32,
    #[serde(rename = "total_cents")] // <- Rust field renamed; wire format stays "total_cents"
    total_amount_cents: u32,
}
```

**Why this way:** `total_amount_cents` is a clearer Rust-side name than
the original `total_cents`, but every stored or already-published JSON
document still uses the old key — `#[serde(rename = "...")]` keeps that
document compatible while letting the Rust API improve, which
[serde's field attribute docs](https://serde.rs/field-attrs.html#rename)
describe as the standard way to decouple the two.

### Scenario: Testing

A round-trip test — serialize a value, deserialize it back, and assert
equality — is the standard way to verify a derived `Serialize`/
`Deserialize` pair actually agree with each other, and it needs
`PartialEq` and `Debug` derived on the type for `assert_eq!` to work at
all.

```
// [dependencies] serde = { version = "1", features = ["derive"] }, serde_json = "1"
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)] // <- Debug + PartialEq needed for assert_eq!
struct SensorReading {
    id: u32,
    celsius: f64,
}

#[test]
fn round_trips_through_json() {
    let original = SensorReading { id: 7, celsius: 21.5 };
    let json = serde_json::to_string(&original).unwrap();
    let restored: SensorReading = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored); // <- confirms serialize/deserialize are true inverses
}
```

**Why this way:** a round-trip assertion catches the common serde mistake
of an attribute (a rename, a custom `with =`) that works in one direction
but not the other; `assert_eq!` needing both `PartialEq` and `Debug` is
the same requirement the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
notes for any equality assertion, serde-derived types included.

## Explanation (Embedded)

`serde`'s traits and its `derive` macro don't themselves need `std` — the
crate builds with `default-features = false` to drop the parts of its
API that assume an allocator, leaving `Serialize`/`Deserialize` and the
derive fully usable in a `#![no_std]` crate. What typically doesn't carry
over is the format crates most tutorials reach for: `serde_json` and
`toml` both produce `String`/`Vec`-shaped output, so they need at least
`alloc`, and in practice assume a hosted environment with plenty of heap.

The embedded-world answer is `postcard` — a binary serde data format
designed specifically for resource-constrained targets: compact (no
field names or JSON punctuation on the wire, just the encoded values),
and able to serialize into a fixed-size buffer without requiring a heap
allocator for that direction. (Its exact allocator requirements differ a
little by API used and by version, so check `postcard`'s own
documentation for the specifics that matter for a given target rather
than assuming every corner of it is allocator-free.) The same
`#[derive(Serialize, Deserialize)]` written for `serde_json` on a hosted
config struct works unchanged with `postcard` on a `#![no_std]` firmware
— it's the format crate underneath, not the derive, that changes.

## Basic usage example (Embedded)

```
// [dependencies] serde = { version = "1", default-features = false, features = ["derive"] }, postcard = "1"
#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)] // <- same derive as the hosted example; the format crate below is what differs
struct SensorReading {
    id: u32,
    millivolts: u16,
}

fn encode(reading: &SensorReading, buf: &mut [u8]) -> usize {
    let used = postcard::to_slice(reading, buf).unwrap(); // <- serializes into a caller-provided buffer, no heap needed
    used.len()
}
```

## Best practices & deeper information (Embedded)

### Scenario: Serializing and deserializing

A sensor node needs to send a compact reading over a low-bandwidth radio
link — `postcard` serializes straight into a fixed-size, statically
sized buffer, with no allocation and no wire overhead spent on field
names the way a JSON payload would carry.

```
// [dependencies] serde = { version = "1", default-features = false, features = ["derive"] }, postcard = "1"
#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SensorReading {
    id: u8,
    celsius_tenths: i16,
}

fn build_packet(reading: &SensorReading) -> [u8; 16] {
    let mut buf = [0u8; 16]; // <- fixed-size, stack-allocated: no heap involved
    let used = postcard::to_slice(reading, &mut buf).unwrap(); // <- postcard: the no_std-friendly wire format
    let len = used.len();
    buf[..len].try_into().unwrap_or(buf)
}
```

**Why this way:** a radio packet has a hard size ceiling and no
allocator to spare, so a format that serializes into a caller-owned
buffer without needing `alloc` fits directly, whereas `serde_json` would
need heap-backed `String`/`Vec` output just to produce the bytes to
send; `postcard`'s own documentation positions it specifically for this
resource-constrained niche.

### Scenario: Designing a public API

A config struct shared between firmware and a desktop configuration tool
should use field types that compile on both sides — a plain `serde`
derive over primitives and fixed-size arrays works unchanged in a
`#![no_std]` build and a hosted one, without special-casing either
target.

```
// [dependencies] serde = { version = "1", default-features = false, features = ["derive"] }
#![no_std]
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DeviceConfig {
    pub sample_rate_hz: u16,
    pub channel_mask: u8, // <- plain primitives: no String/Vec, so this compiles under no_std as-is
}
```

**Why this way:** choosing allocator-free field types up front means the
same struct definition, and the same derive, serializes with `postcard`
on the firmware side and with `serde_json` on a desktop configuration
tool without maintaining two parallel type definitions — `serde`'s
`default-features = false` mode is exactly what keeps that single
definition available to the `no_std` side of that pairing.

---
title: "Derive macros"
area: "Macros & Metaprogramming"
embedded_support: full
groups: ["Macros & Metaprogramming", "Declarative / Metaprogramming", "Generating Code / Metaprogramming", "Macros & Code Generation"]
related_syntax: ["#[derive(...)]", "#[proc_macro] / #[proc_macro_derive(...)] / #[proc_macro_attribute]"]
see_also: ["Declarative macros", "Procedural macros", "Attribute-like macros", "Function-like macros", "Derivable traits"]
---

## Explanation

A derive macro is a function marked `#[proc_macro_derive(Name)]` with
the signature `fn(TokenStream) -> TokenStream`, registered as the
implementation behind `#[derive(Name)]`. When the compiler sees
`#[derive(Name)]` on a struct or enum, it hands that item's tokens —
name, fields, generics, attributes, all of it — to the registered
function, and splices whatever `TokenStream` comes back in as
*additional* code next to the original item. The original struct or enum
is never edited or replaced, only added to.

This page is about that general mechanism, not about which traits exist
to derive — the built-in compiler derives (`Debug`, `Clone`,
`PartialEq`, and the rest of the small fixed set) are covered on
[Derivable traits](../traits-polymorphism/derivable-traits.md). What's
here is the machinery any crate can hook into to add its *own*
derivable trait, which is exactly how third-party derives like serde's
`Serialize`/`Deserialize` work: the same `#[derive(...)]` syntax at the
call site, but backed by a crate-provided `#[proc_macro_derive]`
function instead of a built-in compiler intrinsic.

The mental model: the macro function receives the annotated item as a
`TokenStream`, inspects it (real-world derive macros almost always parse
it into a structured tree with the `syn` crate first), decides what
implementation(s) to generate based on that structure, and builds the
output (almost always with the `quote` crate), returning it as a new
`TokenStream`. Because a derive can only *add* code, never remove or
rewrite the original item, it is structurally incapable of the kind of
transformation an [attribute-like macro](attribute-like-macros.md) can
do — that asymmetry is deliberate, and it's why `#[derive(...)]` feels
safe to sprinkle liberally: it can never change what you wrote, only
generate more code alongside it.

Like every procedural macro (see [Procedural macros](procedural-macros.md)
for the shared crate-splitting rule), a `#[proc_macro_derive]` function
must live in its own crate with `proc-macro = true` — it cannot be
defined and used in the same compilation. A library that wants
`#[derive(Describe)]` to read as part of its own API, the way serde's
derive does, typically re-exports the macro from its main crate rather
than asking downstream users to depend on the internal `-macros`/`-derive`
crate directly.

## Basic usage example

The derive macro's definition, in its own `proc-macro = true` crate:

```
use proc_macro::TokenStream;

#[proc_macro_derive(Describe)] // <- registers this function as the implementation of `#[derive(Describe)]`
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let name = source
        .split_whitespace()
        .skip_while(|&word| word != "struct" && word != "enum")
        .nth(1)
        .expect("derive input is always a struct or enum")
        .to_string(); // a real implementation parses this properly with `syn` instead

    format!("impl Describe for {name} {{ fn describe() -> &'static str {{ \"{name}\" }} }}")
        .parse()
        .unwrap()
}
```

and the consuming crate, which depends on it and defines the trait the
derive is generating an impl for:

```
// Cargo.toml: describe_macros = { path = "../describe_macros" }
use describe_macros::Describe;

trait Describe {
    fn describe() -> &'static str;
}

#[derive(Describe)] // <- expands to `impl Describe for SensorReading { ... }` at compile time
struct SensorReading {
    celsius: f64,
}

fn main() {
    println!("{}", SensorReading::describe());
}
```

**Restriction:** `derive_describe` cannot live in the same crate as the
`#[derive(Describe)]` call above it — a `proc-macro = true` crate can
only export macros, never ordinary items alongside them.

## Best practices & deeper information

### Scenario: Serializing and deserializing

serde's `Serialize`/`Deserialize` derive is the canonical real-world
derive macro: it inspects a struct's fields at compile time and
generates the exact (de)serialization code a human would otherwise
hand-write field by field.

```
// [dependencies] serde = { version = "1", features = ["derive"] }, serde_json = "1"
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)] // <- serde's derive macro expands into full Serialize/Deserialize impls
struct SensorReading {
    id: u32,
    celsius: f64,
}

fn to_json(reading: &SensorReading) -> String {
    serde_json::to_string(reading).unwrap()
}
```

**Why this way:** the derive inspects the struct's actual field names
and types at compile time, so adding or renaming a field never requires
touching hand-written conversion code — this is the core value
proposition [serde.rs](https://serde.rs/derive.html) describes for its
derive macro over writing `Serialize`/`Deserialize` impls by hand.

### Scenario: Designing a public API

Shipping a custom derive as part of a library's public surface means
pairing the trait with its derive macro under the same name and
re-exporting both from one crate, so users never see the internal
proc-macro crate.

```
// describe-macros/Cargo.toml
// [lib]
// proc-macro = true

// describe/Cargo.toml
// [dependencies]
// describe-macros = { path = "../describe-macros", version = "0.1" }

// describe/src/lib.rs
pub trait Describe {
    fn describe() -> &'static str;
}
pub use describe_macros::Describe; // <- re-exports the derive macro under the same name as the trait

// downstream crate
use describe::Describe;

#[derive(Describe)] // <- users see one dependency and one name, never the internal macro crate
struct SensorReading {
    celsius: f64,
}
```

**Why this way:** a trait and its derive macro occupy different
namespaces (type vs. macro), so re-exporting both under one identifier
is legal and reads as a single coherent API item — the same shape
serde's own `Serialize` trait and derive share, which
[Effective Rust](https://effective-rust.com/) points to as the
idiomatic way to ship a derivable trait.

## Explanation (Embedded)

A derive macro is exactly as host-side as every other procedural macro
(see [Procedural macros](procedural-macros.md) for the general
host/target split): `#[proc_macro_derive(...)]` is a plain function that
runs during the *embedded* crate's compilation, on the machine doing the
compiling, and never executes on the microcontroller itself — only the
`TokenStream` it returns, which the compiler splices in as ordinary
source, ends up in the firmware image. That means a derive macro has no
special embedded story at the mechanism level; what matters is whether
the code it generates is itself `no_std`-compatible. Several real derive
macros are written specifically for that constraint: `defmt`'s
`#[derive(Format)]` generates an implementation of `defmt::Format` — a
`no_std`-oriented stand-in for `Debug`/`Display` that encodes values
compactly for transmission over a debug probe, rather than by formatting
a `core::fmt`-built string — and serde's ordinary `#[derive(Serialize,
Deserialize)]` is what crates like `postcard` build on to generate
wire-format code that itself never allocates. In both cases the derive
*author* had to design the generated impl to avoid assuming a heap or
`std::fmt`; the derive mechanism supplying that impl is unchanged.

## Basic usage example (Embedded)

```
#![no_std]

use defmt::Format;

#[derive(Format)] // <- proc-macro derive; runs on the host, generates a no_std-compatible impl
struct SensorReading {
    id: u8,
    celsius: f32,
}

fn log_reading(reading: &SensorReading) {
    defmt::info!("reading: {}", reading); // <- uses the generated Format impl, no core::fmt::Display involved
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL crate's public error type needs to be loggable over RTT without
pulling in `core::fmt`'s heavier formatting machinery; deriving
`defmt::Format` gives it that for free, the same way `#[derive(Debug)]`
would in a hosted crate.

```
#[derive(defmt::Format)] // <- generates a no_std, wire-efficient Format impl, not a core::fmt one
pub enum SensorError {
    NotResponding,
    OutOfRange { celsius: f32 },
}

fn report(err: &SensorError) {
    defmt::warn!("sensor error: {}", err); // <- consumes the derived impl directly
}
```

**Why this way:** a hand-written `core::fmt::Display` impl still routes
through the same formatting machinery `println!` does, which is heavier
than most constrained targets want just for diagnostic logging — the
[`defmt` documentation](https://defmt.ferrous-systems.com/) describes
`Format` as deliberately built to avoid that cost, which is exactly what
the derive gives a HAL's error types for free.

### Scenario: Serializing and deserializing

A sensor packet transmitted over a wire link needs a compact,
allocation-free encoding; deriving serde's `Serialize`/`Deserialize` and
encoding through `postcard` gives that without needing `alloc` at all.

```
// [dependencies] serde = { version = "1", default-features = false, features = ["derive"] }, postcard = "1"
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)] // <- serde's derive; the generated code itself never allocates
struct SensorPacket {
    id: u8,
    celsius: f32,
}

fn encode(packet: &SensorPacket) -> postcard::Result<heapless::Vec<u8, 32>> {
    postcard::to_vec(packet) // <- postcard serializes into a fixed-capacity buffer, no heap involved
}
```

**Why this way:** serde's derive itself makes no allocation decisions —
[serde.rs's `no_std` support notes](https://serde.rs/no-std.html) confirm
the generated code is allocation-agnostic — so the choice of a
heap-free wire format like `postcard` over `serde_json` is what actually
determines whether the encoding step needs an allocator at all.

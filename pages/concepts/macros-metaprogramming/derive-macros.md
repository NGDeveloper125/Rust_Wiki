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

## Embedded Rust Notes

**Full support.** A derive macro runs entirely at compile time and
produces ordinary Rust code, so it has no runtime cost and no `std`
requirement of its own — support depends only on whether the *generated*
code (and any trait bounds it relies on) is `no_std`-friendly. The
`no_std`-oriented logging crate `defmt` ships `#[derive(Format)]` as a
real-world embedded derive macro, generating a `defmt::Format`
implementation the same way `#[derive(Debug)]` would, but wire-efficient
for constrained targets instead of routing through `core::fmt`.

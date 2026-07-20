---
title: "Procedural macros"
area: "Macros & Metaprogramming"
embedded_support: full
groups: ["Macros & Metaprogramming", "Declarative / Metaprogramming", "Generating Code / Metaprogramming", "Macros & Code Generation"]
related_syntax: ["#[proc_macro]", "#[proc_macro_derive]", "#[proc_macro_attribute]"]
see_also: ["Declarative macros", "Derive macros", "Attribute-like macros", "Function-like macros"]
---

## Explanation

"Procedural macro" is the umbrella term for the three compiler-plugin
macro kinds — derive, attribute-like, and function-like. Each one is an
ordinary Rust function with the signature `fn(TokenStream) -> TokenStream`
(attribute-like macros take two token streams), executed by the compiler
during macro expansion, before type-checking, whose return value is
spliced into the surrounding code exactly as if the programmer had
written it there directly.

They exist because [declarative macros](declarative-macros.md) only
match syntax patterns and can't run arbitrary logic or inspect a type's
actual shape. A procedural macro is unrestricted Rust code: it can parse
the incoming tokens into a full syntax tree (in practice almost always
via the `syn` crate), inspect field names, types, and generics, and
build a new tree (almost always via the `quote` crate) before returning
it. This is what makes it possible to write a macro that emits a
different implementation depending on a struct's actual fields — a
`derive` macro's entire job — which a purely pattern-matching macro
cannot do.

The crate-splitting requirement is the single most surprising thing
about procedural macros to anyone coming from `macro_rules!`, where
definition and use can share a file: a function can only be exported as
a procedural macro from a crate whose `Cargo.toml` declares
`proc-macro = true` under `[lib]`, and such a crate may export macros
and nothing else — no ordinary public functions or types alongside them.
A crate cannot invoke a procedural macro that it defines itself; the
macro must be compiled first, as its own separate crate, before any
crate that writes `#[derive(...)]`, `#[some_attribute]`, or
`some_macro!(...)` against it.

The three specific forms differ in what they receive and what they're
allowed to do with it: [derive macros](derive-macros.md)
(`#[proc_macro_derive]`) attach new code alongside an item without
touching the original; [attribute-like macros](attribute-like-macros.md)
(`#[proc_macro_attribute]`) receive their own arguments *and* the
annotated item, and can replace that item entirely; [function-like
macros](function-like-macros.md) (`#[proc_macro]`) are invoked with `!`
like a declarative macro but run arbitrary Rust instead of matching
patterns. All three share the token-stream-in/token-stream-out model
described here — the sibling pages cover what's specific to each.

## Basic usage example

A procedural macro's defining function always lives in its own
`proc-macro = true` crate:

```
use proc_macro::TokenStream;

#[proc_macro] // <- the compiler hands this function raw tokens instead of calling it at runtime
pub fn tuple_of_two(input: TokenStream) -> TokenStream {
    let expr = input.to_string();
    format!("({expr}, {expr})").parse().unwrap()
}
```

and here is a separate, consuming crate that depends on it:

```
// Cargo.toml: proc_macro_demo = { path = "../proc_macro_demo" }
use proc_macro_demo::tuple_of_two;

fn main() {
    let pair = tuple_of_two!(4 + 1); // <- expands at compile time to (4 + 1, 4 + 1)
    println!("{pair:?}");
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A library that wants `#[derive(Reading)]` to feel like part of its own
crate — the way serde's derive does — ships the macro in an internal
crate and re-exports it, so users add one dependency instead of two.

```
// sensorkit-macros/Cargo.toml
// [lib]
// proc-macro = true

// sensorkit-macros/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_derive(Reading)] // <- proc-macro crates may only export macros like this one
pub fn derive_reading(input: TokenStream) -> TokenStream {
    // ... inspect `input`, build an impl, return it as a TokenStream
    TokenStream::new()
}

// sensorkit/Cargo.toml
// [dependencies]
// sensorkit-macros = { path = "../sensorkit-macros", version = "0.1" }

// sensorkit/src/lib.rs
pub use sensorkit_macros::Reading; // <- re-exports the macro so users depend on one crate, not two
```

**Why this way:** `proc-macro = true` crates can only export macros, so
the split is mandatory, not stylistic; re-exporting from the main crate
is exactly the shape serde (`serde`/`serde_derive`) and tokio
(`tokio`/`tokio-macros`) use, and [Effective Rust](https://effective-rust.com/)
recommends hiding that internal split from downstream users.

### Scenario: Testing

A procedural macro only exists at compile time, so its output is tested
the same way any generated code is tested: compile a real use of it in
an integration test and assert on the resulting runtime behavior.

```
// sensorkit/tests/derive_reading.rs — integration test in the crate that USES the proc macro
use sensorkit::Reading;

#[derive(Reading)] // <- the proc macro under test; expansion happens before this test even runs
struct Temperature {
    celsius: f64,
}

#[test]
fn describes_generated_impl() {
    let t = Temperature { celsius: 21.5 };
    assert_eq!(t.describe(), "Temperature"); // exercises the code the macro generated
}
```

**Why this way:** you cannot unit-test macro expansion from inside the
`proc-macro = true` crate that defines it (it can export nothing but
macros), so the
[Rust Book's guidance](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
on integration tests exercising only the public API is the natural fit —
the test lives in the consuming crate and checks what the macro produced.

## Embedded Rust Notes

**Full support.** A procedural macro runs entirely on the host machine
during compilation and produces ordinary Rust source — it has no
runtime footprint and no `std` requirement of its own, so it works
identically when the *generated* code targets `#![no_std]`. This is
exactly the mechanism behind common embedded tooling: `cortex-m-rt`'s
`#[entry]` is an attribute-like macro, and `defmt`'s `#[derive(Format)]`
is a derive macro — both ordinary procedural macros whose *output* is
what has to be `no_std`-friendly, not the macro itself.

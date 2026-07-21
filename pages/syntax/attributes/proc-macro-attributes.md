---
title: "#[proc_macro] / #[proc_macro_derive(...)] / #[proc_macro_attribute]"
kind: attribute
embedded_support: full
groups: ["Macros", "Macros & Metaprogramming"]
related_concepts: ["Procedural macros", "Derive macros", "Attribute-like macros", "Function-like macros"]
related_syntax: ["macro_rules!"]
see_also: ["Procedural macros", "Derive macros", "Attribute-like macros", "Function-like macros"]
---

## Explanation

These three attributes register an ordinary function as a specific kind
of procedural macro. Each requires the crate it's defined in to declare
`proc-macro = true` under `[lib]` in `Cargo.toml`; such a crate may
export only functions tagged with one of these three (plus private
helper items) — no ordinary public functions or types alongside them.
The function itself must be `pub` and return `TokenStream`.

- **`#[proc_macro]`** — registers a **function-like macro**. Signature:
  `fn(TokenStream) -> TokenStream`. Invoked at the call site as
  `name!(...)`, the same syntax as a `macro_rules!` macro. See
  [Function-like macros](../../concepts/macros-metaprogramming/function-like-macros.md)
  for what this kind is for.
- **`#[proc_macro_derive(TraitName)]`** — registers a **derive macro** as
  the implementation behind `#[derive(TraitName)]`. Signature:
  `fn(TokenStream) -> TokenStream`. Optionally takes a second argument,
  `attributes(helper_one, helper_two, ...)`, which reserves one or more
  otherwise-inert helper attribute names (e.g. `#[proc_macro_derive(Reading, attributes(unit))]`
  reserves `#[unit(...)]`) as legal to write inside the annotated item,
  for the derive function itself to inspect via its `TokenStream` input.
  See [Derive macros](../../concepts/macros-metaprogramming/derive-macros.md).
- **`#[proc_macro_attribute]`** — registers an **attribute-like macro**.
  Signature: `fn(TokenStream, TokenStream) -> TokenStream` — the
  attribute's own arguments, then the full annotated item. Invoked as
  `#[name]` or `#[name(...)]` above an item. See
  [Attribute-like macros](../../concepts/macros-metaprogramming/attribute-like-macros.md).

A given function may carry only one of these three; a single
`proc-macro = true` crate is free to export any mix of them, each its
own separately tagged function. This page covers only the registration
syntax itself — see [Procedural macros](../../concepts/macros-metaprogramming/procedural-macros.md)
for the shared token-stream-in/token-stream-out mental model, and the
kind-specific concept pages above for what each macro kind is actually
for and how it's used well.

## Basic usage example

```
use proc_macro::TokenStream;

#[proc_macro] // <- registers a function-like macro, invoked as `build!(...)`
pub fn build(input: TokenStream) -> TokenStream { input }

#[proc_macro_derive(Reading)] // <- registers a derive macro, invoked as `#[derive(Reading)]`
pub fn derive_reading(input: TokenStream) -> TokenStream { TokenStream::new() }

#[proc_macro_attribute] // <- registers an attribute-like macro, invoked as `#[traced]`
pub fn traced(_attr: TokenStream, item: TokenStream) -> TokenStream { item }
```

## Best practices & deeper information

### Scenario: Designing a public API

A macro crate for a sensor-data library offers all three forms from one
`proc-macro = true` crate: a derive for boilerplate trait impls, an
attribute for wrapping handler functions, and a function-like macro for
validated literals — each function tagged with the registration
attribute matching what it needs to receive.

```
// sensorkit-macros/Cargo.toml
// [lib]
// proc-macro = true

use proc_macro::TokenStream;

#[proc_macro_derive(Reading, attributes(unit))] // <- also reserves the inert helper attribute #[unit(...)]
pub fn derive_reading(input: TokenStream) -> TokenStream {
    TokenStream::new() // a real implementation inspects `input`'s fields here
}

#[proc_macro_attribute] // <- receives the attribute's own args AND the annotated item, as two streams
pub fn traced(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro] // <- receives only the tokens written inside its own `!(...)` invocation
pub fn sql(input: TokenStream) -> TokenStream {
    input // a real implementation validates the embedded SQL text here
}
```

**Why this way:** each registration attribute fixes the function's exact
signature and what it's invoked as, so choosing between them is really
choosing which of the three token-stream contracts a given macro needs —
the [Rust Reference's procedural macros chapter](https://doc.rust-lang.org/reference/procedural-macros.html)
documents all three signatures, and a single `proc-macro = true` crate
is free to mix any number of each, since the registration is per
function, not per crate.

## Embedded Rust Notes

**Full support.** Registration and execution both happen entirely on the
host machine at compile time and produce ordinary Rust source, so none
of these attributes have a runtime footprint or `std` requirement of
their own — support depends only on whether the *generated* code targets
`#![no_std]`. Real embedded tooling uses all three: `cortex-m-rt`'s
`#[entry]` is `#[proc_macro_attribute]`-backed, and `defmt`'s
`#[derive(Format)]` is `#[proc_macro_derive]`-backed.

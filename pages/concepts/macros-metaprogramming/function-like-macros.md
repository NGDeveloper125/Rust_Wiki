---
title: "Function-like macros"
area: "Macros & Metaprogramming"
embedded_support: full
groups: ["Macros & Metaprogramming", "Generating Code / Metaprogramming", "Macros & Code Generation"]
related_syntax: ["#[proc_macro] / #[proc_macro_derive(...)] / #[proc_macro_attribute]", "!", "macro_rules!"]
see_also: ["Declarative macros", "Procedural macros", "Derive macros", "Attribute-like macros"]
---

## Explanation

A function-like macro is a function marked `#[proc_macro]` with the
signature `fn(TokenStream) -> TokenStream`, invoked the same way a
[declarative macro](declarative-macros.md) is — `name!(...)` — but
instead of the compiler pattern-matching the call's tokens against
hand-written arms, the entire token stream inside the invocation's
delimiters is handed to the function as-is, free to run arbitrary Rust
code to decide what comes back.

`macro_rules!` is still the default choice for a `name!(...)`-style
invocation, since it needs no separate crate and is hygienic
automatically at no extra cost. A `#[proc_macro]` earns its
crate-splitting overhead when the transformation genuinely needs more
than matching token shapes: parsing and validating a non-Rust
mini-language written inside the parentheses (a SQL string, a regex, a
templated format string) at compile time, or performing lookups and
computation a fixed set of `macro_rules!` arms simply can't express.

A function-like macro isn't attached to an existing item, so it can't
inspect or transform "the function below it" the way an
[attribute-like macro](attribute-like-macros.md) does, nor does it
automatically receive a struct's shape the way a
[derive macro](derive-macros.md) does — it only ever sees the tokens
written literally between its own invocation's delimiters. That makes it
the right tool specifically for standalone, expression- or item-position
code generation (building a literal, validating a format, generating a
lookup table), not for reshaping something else.

Like every procedural macro (see [Procedural macros](procedural-macros.md)
for the shared crate-splitting rule), a `#[proc_macro]` function must be
defined in its own `proc-macro = true` crate, compiled before, and
separately from, any crate that writes `name!(...)` against it.

## Basic usage example

The function-like macro's definition, in its own `proc-macro = true`
crate:

```
use proc_macro::TokenStream;

#[proc_macro] // <- invoked as `shout!("...")`; the compiler passes it raw tokens, not a runtime call
pub fn shout(input: TokenStream) -> TokenStream {
    input.to_string().to_uppercase().parse().unwrap() // <- re-parses the uppercased text back into tokens
}
```

and the consuming crate that invokes it:

```
// Cargo.toml: shout_macro = { path = "../shout_macro" }
use shout_macro::shout;

fn main() {
    println!("{}", shout!("system ready")); // <- expands at compile time to the literal "SYSTEM READY"
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A function-like macro can validate a SQL literal's shape at compile
time, turning a class of runtime typo bugs into compile errors that a
plain string argument could never catch.

```
// crate "querykit" defines `sql` as a #[proc_macro]
use querykit::sql;

fn recent_readings_query(limit: u32) -> String {
    // AVOID: a plain string literal compiles even with a typo, and fails only at runtime
    // let base = "SELCT id, celsius FROM readings";

    let base = sql!("SELECT id, celsius FROM readings ORDER BY id DESC"); // PREFER: rejected at compile time if malformed
    format!("{base} LIMIT {limit}")
}
```

**Why this way:** inspecting a literal's shape at compile time is
exactly what a plain function cannot do but a function-like proc macro
can, since the macro sees the literal's raw tokens before the program
ever runs — [Effective Rust](https://effective-rust.com/) cites
compile-time validation of embedded mini-languages as the strongest
justification for a function-like macro over a normal function call.

### Scenario: Working with collections

A map-literal macro gives `HashMap` the same ergonomic construction
syntax `vec!` already gives `Vec`, expanding a `key => value` list into
constructor calls at compile time.

```
// crate "collectionkit" defines `hashmap` as a #[proc_macro]
use collectionkit::hashmap;

fn alert_thresholds() -> std::collections::HashMap<&'static str, f64> {
    hashmap! { // <- function-like macro: expands into HashMap::from([...]) at compile time
        "low" => 10.0,
        "normal" => 20.0,
        "high" => 35.0,
    }
}

fn main() {
    let thresholds = alert_thresholds();
    assert_eq!(thresholds["normal"], 20.0);
}
```

**Why this way:** this mirrors how `vec![]` turns a literal list into
constructor calls the compiler already knows how to optimize, giving map
literals the same construction ergonomics collections like `Vec` get
natively — the
[Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/data_structures.html)
shows the equivalent hand-written pattern this kind of macro is built to
shorten.

## Embedded Rust Notes

**Full support.** A function-like macro runs entirely at compile time
and produces ordinary Rust code, so it has no runtime footprint and no
`std` requirement of its own — support depends only on whether the
generated code is `no_std`-friendly. Embedded crates use this pattern
for things like compile-time-checked register addresses or bit-pattern
literals, where validating a value's shape before it's flashed to a
device is far cheaper than discovering the mistake on hardware.

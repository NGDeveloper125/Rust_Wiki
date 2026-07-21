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

## Explanation (Embedded)

A function-like macro's `#[proc_macro]` function runs on the host at
compile time exactly like any other procedural macro (see [Procedural
macros](procedural-macros.md)) — it never touches the target, and only
the `TokenStream` it returns has to be ordinary, already-`no_std`-
compatible Rust for the embedded build to succeed; the macro's own
implementation is free to use full `std` regardless of what the
consuming crate targets. Where this shape earns its crate-splitting cost
in embedded code is compile-time validation of values that would
otherwise only be checked once flashed to hardware: a peripheral base
address that must land on a word boundary, a CAN identifier that must
fit inside 11 bits, a checksum byte that must match a computed value —
anything a fixed set of `macro_rules!` arms can't reject by pattern shape
alone, because rejecting it requires actually computing something. The
pattern generalized as `singleton!` in crates like `cortex-m` is a
related but distinct use: rather than validating a literal, it's a
function-like macro that expands to a static paired with take-once
semantics at runtime, so a `'static mut` reference can be handed out
safely exactly once — a shape that's awkward to write by hand for every
buffer that needs it, and that a macro can generate uniformly instead.
As with `vec!`/`format!` (see [`vec!`](../../syntax/macros/vec-macro.md)
and [`write!`/`writeln!`](../../syntax/macros/write-macros.md)), the one
caveat worth designing around deliberately is whether the macro's
expansion assumes a heap: a function-like macro that expands to a `Vec`
or `String` literal inherits the same `alloc`-plus-allocator requirement
those macros document, while one that expands to a fixed-size array or a
`heapless` type avoids that requirement entirely.

## Basic usage example (Embedded)

```
use proc_macro::TokenStream;

#[proc_macro] // <- ordinary std-using Rust: compiled for, and run on, the host — never the target
pub fn reg_addr(input: TokenStream) -> TokenStream {
    let text = input.to_string();
    let value = usize::from_str_radix(text.trim_start_matches("0x"), 16).unwrap();
    assert!(value % 4 == 0, "peripheral base address must be 4-byte aligned"); // <- fails the build, not the flash
    text.parse().unwrap()
}
```

and the consuming, `#![no_std]` firmware crate that invokes it:

```
// Cargo.toml: regkit = { path = "../regkit" }
use regkit::reg_addr;

const GPIOA_BASE: usize = reg_addr!(0x4800_0000); // <- misalignment would fail to compile, long before any flashing
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A CAN bus driver accepts identifiers as compile-time literals; a
function-like macro rejects any literal that doesn't fit the 11-bit
standard identifier range before the firmware ever runs, instead of
silently truncating it at runtime.

```
// crate "cankit" defines `can_id` as a #[proc_macro]
use cankit::can_id;

const HEARTBEAT_ID: u16 = can_id!(0x7FF); // <- rejected at compile time if it doesn't fit 11 bits
```

**Why this way:** a plain `u16` constant would accept and silently mask
any out-of-range value, turning a protocol violation into a bug that
only shows up on the bus — [Effective Rust](https://effective-rust.com/)
cites exactly this kind of compile-time validation of a domain-specific
literal as the strongest case for a function-like macro over a plain
constant.

### Scenario: Designing a public API

A calibration table needs to be macro-generated from a short list of
readings, but expanding it into a `Vec` would drag `alloc` and a
`#[global_allocator]` into every crate that just wants a handful of
constants.

```
// crate "calkit" defines `lookup_table` as a #[proc_macro]
use calkit::lookup_table;

// AVOID: expanding to `vec![...]` would require `alloc` + a #[global_allocator] just to hold constants
// PREFER: expanding to a plain array needs neither, and the table lives in .rodata
const GAIN_CURVE: [u16; 4] = lookup_table!(100, 205, 410, 820); // <- macro-generated, but an ordinary const array
```

**Why this way:** the same `alloc`-vs-heapless design choice documented
for [`vec!`](../../syntax/macros/vec-macro.md) and
[`write!`](../../syntax/macros/write-macros.md) applies to any macro a
crate author writes, not just the standard library's — expanding to a
fixed-size array sidesteps the allocator requirement altogether rather
than merely offering a `heapless` alternative alongside it.

---
title: "!"
kind: operator
embedded_support: full
groups: [Logical, Basics, "Macros & Metaprogramming"]
related_concepts: [Operator overloading]
related_syntax: ["!="]
see_also: []
---

## Explanation

As a prefix operator, `!` is logical NOT on `bool` and bitwise complement
on integers (e.g. `!0b1010u8` flips every bit), overloadable via
`std::ops::Not`.

After a path (like `println` or `vec`) and followed by a delimited group
of tokens, `!` instead marks a **macro invocation** — a completely
unrelated meaning, as in `println!("hi")` or `vec![1, 2, 3]`.

`ident!` is not the `Not` trait applied to `ident`; it's the macro-call
syntax. The distinction is purely positional — a `!` following a path and
followed by a `(...)`/`[...]`/`{...}` group is a macro invocation, while
a `!` starting an expression is prefix negation (whitespace doesn't
matter: `println ! ("hi")` compiles fine).

`!` alone (no operand, in type position) is also the **never type**
(`fn diverges() -> !`) — the type of an expression that never produces a
value, such as `return`, `break`, `panic!()`, or an infinite `loop` with
no `break`.

## Usage examples

### Negating a boolean value

```
let done = false;
let not_done = !done; // <- `!` negates the bool
```

### Validating input

Writing a guard as `if !is_valid(x)` keeps the happy path as the
unindented continuation of the function, instead of nesting the whole
body inside `if is_valid(x) { ... }`.

```
struct Config {
    port: u16,
    hostname: String,
}

fn is_valid(config: &Config) -> bool {
    config.port > 0 && !config.hostname.is_empty()
}

fn load(config: &Config) {
    if !is_valid(config) { // <- `!` negates the validity check into a guard condition
        panic!("invalid config: {}:{}", config.hostname, config.port);
    }
    println!("loading {}:{}", config.hostname, config.port);
}
```

The early-return/early-panic style — guard clauses
that reject bad input up front instead of deeply nesting the success
path inside a positive `if` — is what `!` enables here: negating the
validity check lets the failure case exit immediately, leaving the rest
of the function as the unindented success path.

## Embedded Rust Notes

**Full support.** `Not` lives in `core::ops`; macro invocation and the
never type `!` are core grammar — none of this depends on `std`. Note
that `panic!()` itself behaves differently under `#![no_std]` (see the
[panic! macro](../macros/panic-macro.md) and
[Panic & unwinding](../../concepts/error-handling/panic-and-unwinding.md) pages) — it
still expands via this same `!` syntax, but requires a `#[panic_handler]`
function since there's no default one without `std`.

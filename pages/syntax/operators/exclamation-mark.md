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

## Explanation (Embedded)

All three meanings of `!` carry over into `#![no_std]` unchanged: logical
NOT and bitwise complement are core-language operators overloadable via
`core::ops::Not`, macro-invocation syntax is resolved entirely at compile
time regardless of target, and the never type `!` is likewise a
core-language type with no runtime component. The never type is the one
place `!` becomes *more* prominent rather than merely unchanged. Firmware's
`fn main` conventionally never returns — there's no OS to return control
to — so declaring it `fn main() -> !` and driving it with an unconditional
`loop { }` isn't a stylistic quirk, it's the normal, expected shape of an
embedded entry point, and the compiler uses that same `-> !` signature to
statically confirm the function really never falls off the end. The same
requirement applies to a `#[panic_handler]` function, which likewise must
return `!` since a firmware panic has nowhere to unwind to.

## Usage examples (Embedded)

### Negating a peripheral's ready flag before polling

```
fn poll_until_ready(is_ready: impl Fn() -> bool) {
    while !is_ready() { // <- `!` negates the ready check into a "keep waiting" condition
        // spin until the peripheral reports ready
    }
}
```

### An embedded entry point that never returns

```
#[no_mangle]
fn main() -> ! { // <- `!`: main never returns control to anything, so its return type is the never type
    loop {
        // service peripherals, feed a watchdog, etc. — forever
    }
}
```

### A panic handler, also required to diverge

```
use core::panic::PanicInfo;

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! { // <- `!`: a firmware panic has nowhere to unwind to, so this must diverge too
    loop {} // e.g. halt the core, or reset the microcontroller
}
```

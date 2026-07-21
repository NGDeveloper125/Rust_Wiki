---
title: "ident!(...) / ident!{...} / ident![...]"
kind: macro
embedded_support: full
groups: ["Macro Definition Syntax", "Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)", "Function-like macros"]
related_syntax: ["!", "macro_rules!", "$(...)…"]
see_also: ["!"]
---

## Explanation

Every macro invocation — declarative or procedural, function-like or a
`macro_rules!`-style call — is a path, a `!`, and one delimited group of
tokens. That group may be wrapped in `(...)`, `[...]`, or `{...}`. To the
compiler these three are **100% interchangeable**: `vec!(1, 2, 3)`,
`vec![1, 2, 3]`, and `vec!{1, 2, 3}` all parse identically and expand
identically. This page is about that choice of bracket; see
[`!`](../operators/exclamation-mark.md) for the marker itself and what
makes something a macro invocation in the first place.

Since the delimiters carry no grammatical meaning, the choice between
them is pure convention, guided by what the expansion reads like:

- **`(...)`** — for macros that read like a function call or produce a
  single expression: `println!("...")`, `format!(...)`, `assert_eq!(a, b)`,
  `matches!(value, Pattern)`.
- **`[...]`** — for macros that produce something list- or array-like:
  `vec![1, 2, 3]` is the standard example, echoing the `[1, 2, 3]` array
  literal syntax it's building on top of.
- **`{...}`** — for macros whose expansion is one or more whole items or
  statements rather than a single value: `macro_rules! name { ... }`
  itself, and item-producing macros like std's `thread_local! { ... }`.

The one place the choice has an actual (if narrow) grammatical
consequence: when a macro invocation is used as a statement or item, a
`{...}`-delimited invocation does **not** require a trailing `;` — the
closing brace already ends it, the same rule that lets a `fn` or `impl`
block go without one. A `(...)`- or `[...]`-delimited invocation used as
a statement still needs the trailing `;` (`println!("ready");`).

## Usage examples

### Equivalent delimiters for a list-like macro

```
let a = vec!(1, 2, 3); // <- legal: () instead of the conventional [] for this list-like macro
let b = vec![1, 2, 3]; // <- idiomatic form for vec!, matching array-literal syntax
assert_eq!(a, b);      // <- both invocations expand identically
```

### Designing a public API

A metrics module defines a thread-local counter with std's
`thread_local!` macro, which expands into full item definitions — so, by
convention, it's written with `{...}` and, like any other item, needs no
trailing semicolon.

```
use std::cell::RefCell;

thread_local! { // <- {} form: this macro's expansion is item(s), not a single value
    static REQUEST_COUNT: RefCell<u32> = RefCell::new(0);
} // no trailing `;` needed here, exactly like a `fn` or `impl` block

fn record_request() {
    REQUEST_COUNT.with(|count| *count.borrow_mut() += 1); // <- () form: reads like a method call
}
```

The compiler would accept any of the three delimiter
pairs here, but `{...}` is the idiomatic signal that a macro's expansion
is itself one or more items rather than a value — the same visual cue an
ordinary `mod` or `impl` block gives, which is why
[`thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html)
and `macro_rules!` itself are always written this way in practice.

## Explanation (Embedded)

Delimiter choice is resolved entirely at parse time and carries no
runtime behavior at all, so it's identical under `#![no_std]` — there's
no embedded-specific grammar here. The same conventions from the classic
explanation carry over directly: embedded crates reach for `{...}` when
a macro's expansion is one or more items (`bitflags!`-style register-flag
definitions, `macro_rules!` itself), `(...)` for anything that reads
like a function call or single expression (`defmt::info!(...)`,
`nb::block!(...)`), and `[...]` mainly for list-like expansions, which
are rarer in `no_std` code simply because the standard list-like macro,
`vec!`, needs `alloc` (see [`vec!`](vec-macro.md)).

## Usage examples (Embedded)

### Defining register flags with the {} item form

```
bitflags::bitflags! { // <- {} form: this macro's expansion is a whole struct item, not a single value
    struct StatusFlags: u8 {
        const READY   = 0b0000_0001;
        const TX_BUSY = 0b0000_0010;
        const RX_FULL = 0b0000_0100;
    }
}
```

### Blocking on a non-blocking peripheral operation with the () form

```
let byte = nb::block!(serial.read()); // <- () form: reads like an ordinary function call, evaluates to one value
```

### Same interchangeability, an unconventional [] call

```
let ready = assert_eq![gpio.read_pin(5), true]; // <- legal: [] instead of the conventional () for this expression-like macro
```

---
title: "$(...)…"
kind: macro
embedded_support: full
groups: ["Macro Definition Syntax", "Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)"]
related_syntax: ["macro_rules!", "$ident", "$ident:kind"]
see_also: ["$ident:kind"]
---

## Explanation

`$(...)` groups a sub-pattern that can repeat, in both the matcher and
the transcriber of a `macro_rules!` arm. Immediately after the closing
`)` comes an optional **separator token** (almost always `,`, but any
single token is legal), followed by exactly one repetition count:

- **`*`** — zero or more repetitions.
- **`+`** — one or more repetitions; an invocation with none fails to
  match this arm.
- **`?`** — zero or one repetition; `?` cannot take a separator, since
  there's never a second repetition to separate from.

In a **matcher**, `$($item:expr),*` matches zero or more expressions,
each one separated from the next by a literal `,` in the input tokens,
and captures each match of `$item` as its own repetition. In the
**transcriber**, `$($item),*` re-emits the enclosed template once per
captured repetition, splicing the separator token back in between
copies — this is the exact mechanism `vec!` itself is built on: matching
`$($x:expr),*` accepts any number of comma-separated elements, and the
expansion re-emits one piece of construction code per element.

The separator used when matching doesn't have to be the one re-emitted
when expanding — a matcher can require commas between inputs while the
transcriber joins the emitted pieces with `+`, `;`, or nothing at all,
since the transcriber's repetition is a separate declaration from the
matcher's. Repetitions can also nest (`$($($inner:tt),*);*`) to match
lists of lists, as long as each metavariable is only ever referenced
inside a transcriber repetition whose depth matches how it was captured.

## Basic usage example

```
macro_rules! sum_all {
    ($($n:expr),*) => { // <- `*`: matches zero or more comma-separated expressions
        0 $(+ $n)* // <- re-emits each captured $n, prefixed with `+`, once per repetition
    };
}

let total = sum_all!(1, 2, 3); // <- expands to 0 + 1 + 2 + 3
```

## Best practices & deeper information

### Scenario: Working with collections

A batch-construction macro needs to accept any number of comma-separated
readings, including an optional trailing comma — exactly the shape std's
own `vec!` is implemented with internally.

```
macro_rules! reading_batch {
    ($($value:expr),* $(,)?) => { // <- `*` for the list, `$(,)?` allows one optional trailing comma
        vec![$($value),*] // <- re-emits each captured value, comma-separated, into a real Vec
    };
}

let batch = reading_batch!(21.4, 19.8, 23.1,); // <- trailing comma accepted thanks to the `?` repetition
```

**Why this way:** std's own `vec!` macro is implemented with this exact
`$($x:expr),* $(,)?` shape, which is why both `vec![1, 2, 3]` and
`vec![1, 2, 3,]` compile — see the
[standard library's `vec!` documentation](https://doc.rust-lang.org/std/macro.vec.html)
for the macro this pattern mirrors; a custom variadic constructor macro
copies the same shape so it tolerates trailing commas the same way
callers already expect from `vec!`.

## Embedded Rust Notes

**Full support.** Repetition is resolved entirely at compile time and
produces ordinary, already-expanded Rust code, so it costs nothing at
runtime and works identically under `#![no_std]`. It's a heavily used
tool in embedded HAL crates specifically: generating one nearly
identical `impl` per GPIO pin or timer instance from a single
`$($pin:ident),*`-style macro call is far less error-prone than
hand-writing dozens of copies.

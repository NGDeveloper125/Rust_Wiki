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

## Usage examples

### Matching and re-emitting zero or more repetitions

```
macro_rules! sum_all {
    ($($n:expr),*) => { // <- `*`: matches zero or more comma-separated expressions
        0 $(+ $n)* // <- re-emits each captured $n, prefixed with `+`, once per repetition
    };
}

let total = sum_all!(1, 2, 3); // <- expands to 0 + 1 + 2 + 3
```

### Working with collections

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

std's own `vec!` macro is implemented with this exact
`$($x:expr),* $(,)?` shape, which is why both `vec![1, 2, 3]` and
`vec![1, 2, 3,]` compile — see the
[standard library's `vec!` documentation](https://doc.rust-lang.org/std/macro.vec.html)
for the macro this pattern mirrors; a custom variadic constructor macro
copies the same shape so it tolerates trailing commas the same way
callers already expect from `vec!`.

## Explanation (Embedded)

Repetition is resolved entirely at compile time and produces ordinary,
already-expanded Rust code, so `$(...)*`/`$(...)+`/`$(...)?` cost nothing
at runtime and work identically under `#![no_std]` — nothing about
matching or re-emitting a repeated sub-pattern depends on a heap, an
allocator, or `std` being present. It's one of the most heavily used
tools in embedded HAL crates specifically, because so much embedded
boilerplate is genuinely repetitive: one nearly identical register
accessor per peripheral instance, one nearly identical GPIO pin type per
physical pin, one nearly identical timer `impl` per timer instance.
Writing `$($pin:ident => $bit:expr),*` once and having it expand into
dozens of hand-equivalent functions is both far less error-prone and far
less to review than the hand-written copies it replaces — this is
exactly the technique tools like `svd2rust` lean on to generate an entire
chip's register API from its hardware description file.

## Usage examples (Embedded)

### Generating GPIO pin accessor functions

```
macro_rules! gpio_pins {
    ($($name:ident => $bit:expr),* $(,)?) => { // <- `*` over pin definitions, trailing comma tolerated
        $(
            fn $name(gpio_odr: &mut u32) { // <- re-emits one function per repetition
                *gpio_odr |= 1 << $bit;
            }
        )*
    };
}

gpio_pins! {
    set_pa5 => 5,
    set_pa6 => 6,
    set_pa7 => 7,
}
// <- expands to three separate functions, set_pa5/set_pa6/set_pa7, one per repetition
```

### Generating register-accessor functions for multiple peripheral instances

A driver needs a `read_status` function for each of several identical
UART instances sitting at different base addresses — exactly the "one
nearly identical accessor per instance" case repetition exists for.

```
macro_rules! uart_status_readers {
    ($($fn_name:ident @ $base:expr),* $(,)?) => {
        $(
            fn $fn_name() -> u32 {
                unsafe { core::ptr::read_volatile(($base + 0x00) as *const u32) } // <- status register at offset 0x00
            }
        )*
    };
}

uart_status_readers! {
    uart1_status @ 0x4001_3800,
    uart2_status @ 0x4000_4400,
    uart3_status @ 0x4000_4800,
}
```

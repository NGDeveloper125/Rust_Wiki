---
title: "format!"
kind: macro
embedded_support: partial
groups: ["Macros & Metaprogramming"]
related_concepts: ["String formatting (Display, Debug, format!)"]
related_syntax: ["println! / print! / eprintln! / eprint!", "write! / writeln!"]
see_also: ["println! / print! / eprintln! / eprint!", "write! / writeln!"]
---

## Explanation

`format!` takes a format string plus arguments and returns a freshly
allocated `String` — the same `{}`/`{:?}` placeholder grammar used by
`println!`, `eprintln!`, and `write!` all bottom out in this one
mini-language, and this page is its canonical home; the other formatting
macros link back here rather than re-deriving it.

Inside `{}`, an argument is selected three ways: **positional**, by index
(`{0}`, `{1}`, referring back to arguments by position, which allows
reuse and reordering); **named**, by writing `{name}` and either passing
`name = value` as an argument or, since Rust 2021, capturing a variable
of that name directly from the enclosing scope (`format!("{order_id}")`
needs no separate `order_id = order_id`); and **implicit**, the plain
`{}` that consumes the next unused positional argument in sequence.
Writing `{:?}` instead of `{}` selects `Debug` rather than `Display` for
that argument (`{:#?}` asks for `Debug`'s pretty, multi-line form).

After the argument selector, an optional `:` introduces formatting
specifiers: a fill character and alignment (`{:>10}` right-aligns in a
10-wide field, `{:^10}` centers, `{:<10}` left-aligns, with a custom fill
character: `{:*>10}`), a sign flag (`{:+}` forces the sign on positive
numbers), a width (`{:10}`), a precision (`{:.2}`, truncating a string or
rounding a float to 2 decimal places — width and precision combine as
`{:>10.2}`), and radix flags (`{:x}`/`{:X}` hex, `{:o}` octal, `{:b}`
binary, `{:e}` scientific notation). Width and precision can themselves be
arguments rather than literals (`{:.prec$}`, `{:width$.prec$}`), letting
the formatting itself be computed at runtime rather than fixed in the
string.

## Basic usage example

```
let order_id = 42;
let amount = 19.9;
let receipt = format!("Order #{order_id}: ${amount:.2}"); // <- named capture + 2-decimal precision, returns an owned String
```

**Restriction:** `format!` (and every macro sharing this grammar) checks
argument counts and placeholder names against the arguments at compile
time — an unused argument, or a `{missing}` name not in scope, is a
compile error rather than a runtime surprise, since the format string
itself must be a string literal (or a `concat!` of literals) known at
compile time, never a runtime-computed `String`.

## Best practices & deeper information

### Scenario: Working with text

Building a fixed-width shipment label combining several fields is exactly
what `format!` is for, instead of repeated `push_str` calls glued
together by hand.

```
struct Shipment {
    tracking_id: String,
    carrier: String,
    weight_kg: f64,
}

fn label(shipment: &Shipment) -> String {
    format!(
        "{:<12} | {:>9} | {:>6.2} kg", // <- left-align id, right-align carrier, 2-decimal weight
        shipment.tracking_id, shipment.carrier, shipment.weight_kg
    )
}

let line = label(&Shipment { tracking_id: "TRK-8841".into(), carrier: "Northwind".into(), weight_kg: 3.4 });
println!("{line}");
```

**Why this way:** the
[std fmt docs](https://doc.rust-lang.org/std/fmt/index.html#width) treat
width, alignment, and precision specifiers as the idiomatic way to build
fixed-width tabular text, replacing manual padding logic with a
declarative format string.

### Scenario: Handling and propagating errors

Constructing a descriptive error message that embeds the offending value
happens at the point of failure, then gets wrapped into a custom error
type so the context isn't lost by the time it surfaces.

```
#[derive(Debug)]
struct ConfigError(String);

fn parse_port(raw: &str) -> Result<u16, ConfigError> {
    raw.trim()
        .parse()
        .map_err(|_| ConfigError(format!("invalid port value: {raw:?}"))) // <- {:?} quotes the raw string in the message
}
```

**Why this way:** the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
and [Effective Rust](https://effective-rust.com/) both favor errors that
carry enough context to diagnose the failure without re-running the
program — building the message with `format!` right where the bad input
is still in scope is how that context gets captured before it's lost.

## Embedded Rust Notes

**Partial support.** The formatting engine itself (`core::fmt`,
`Display`/`Debug`, the `format_args!` machinery this macro expands into)
is pure `core` and works with no allocator at all — what `format!`
specifically needs is `alloc`, since it allocates and returns an owned
`String`. On a `#![no_std]` target with `extern crate alloc` and a
configured `#[global_allocator]`, `format!` works normally; without an
allocator, the usual substitute is `write!`ing into a stack buffer or a
`heapless::String` instead of allocating a fresh one (see
[`write!` / `writeln!`](write-macros.md)).

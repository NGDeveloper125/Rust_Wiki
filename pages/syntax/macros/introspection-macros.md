---
title: "concat! / stringify! / line! / column! / file! / module_path!"
kind: macro
embedded_support: full
groups: ["Compile-time Introspection", "Macros & Metaprogramming"]
related_concepts: []
related_syntax: ["assert! / assert_eq! / assert_ne!", "#[track_caller]"]
see_also: ["assert! / assert_eq! / assert_ne!", "#[track_caller]"]
---

## Explanation

Six small macros, each producing a compile-time value from source text or
build position rather than from any runtime computation:

- `concat!(a, b, ...)` concatenates literal arguments (string, char,
  numeric, boolean literals) into a single string literal, entirely at
  compile time — it cannot take arbitrary runtime expressions the way
  `format!` can; every argument must itself be a literal.
- `stringify!(tokens)` turns a sequence of tokens back into a string
  containing their own source text, without evaluating or type-checking
  them as an expression — `stringify!(1 + 1)` produces the string `"1 +
  1"`, not `"2"`. This is exactly what powers `assert!`'s default failure
  message (see
  [`assert!` / `assert_eq!` / `assert_ne!`](assert-macros.md)): the
  condition passed to `assert!` is captured with `stringify!` so the
  panic message can show the reader the literal source of the check that
  failed.
- `line!()`, `column!()`, and `file!()` each expand to a compile-time
  constant giving the current source location — the line number, column
  number, and file path of the macro invocation itself, respectively.
- `module_path!()` expands to a string literal of the current module's
  dotted path (e.g. `"myapp::handlers::orders"`), as seen from the crate
  root.

All six are commonly composed together inside other macros to build
informative diagnostic or log messages that name their own call site —
`line!()`/`file!()`/`module_path!()` give the "where," `stringify!` gives
the "what," `concat!` joins the pieces into one literal. For call-site
information tied to a specific *function* rather than the macro
invocation itself, [`#[track_caller]`](../attributes/track-caller-attribute.md)
plus `std::panic::Location::caller()` is the runtime equivalent used by
`unwrap()` and similar methods to report where they were called from.

## Usage examples

### Capturing source text and concatenating literals

```
let expr_text = stringify!(1 + 2);            // <- "1 + 2", the source text, not the evaluated "3"
let banner = concat!("build ", "v", "1.0.3"); // <- joined into one &'static str at compile time
```

### Testing

A small custom assertion macro reports the failed condition's own source
text using `stringify!`, the same technique the standard `assert!` macro
uses internally.

```
macro_rules! check_positive {
    ($value:expr) => {
        if $value <= 0 {
            panic!(
                "expected {} to be positive, got {}",
                stringify!($value), // <- turns the *expression itself* into a string, e.g. "order.total"
                $value
            );
        }
    };
}

struct Order { total: i64 }
let order = Order { total: -5 };
check_positive!(order.total); // panics: "expected order.total to be positive, got -5"
```

Reporting the expression's own source text
(`"order.total"`) rather than a hand-written label keeps the message
accurate automatically as the macro is reused at different call sites —
the same trick the
[std docs](https://doc.rust-lang.org/std/macro.stringify.html) describe
`assert!` itself as relying on for its default panic message.

### Designing a public API

A lightweight logging macro tags every log line with its own source
location, using `file!()`/`line!()`/`module_path!()` rather than
requiring the caller to pass that information manually.

```
macro_rules! log_here {
    ($message:expr) => {
        println!(
            "[{}:{} ({})] {}",
            file!(), line!(), module_path!(), $message
            // <- all three are compile-time constants naming the call site of THIS invocation
        )
    };
}

fn process_order(order_id: u64) {
    log_here!(format!("processing order {order_id}"));
}
```

Baking the call site into the macro itself means every
call site gets accurate location info for free without the caller having
to supply it — the same underlying technique logging crates in the
ecosystem build their own logging macros on top of, per the
[std docs](https://doc.rust-lang.org/std/macro.file.html) for
`file!`/`line!`/`module_path!`.

## Embedded Rust Notes

**Full support.** All six are resolved entirely at compile time against
source text and build metadata — none has any runtime dependency on
`std` or an OS, so they work identically, with zero cost, under
`#![no_std]`.

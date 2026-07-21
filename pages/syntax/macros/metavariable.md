---
title: "$ident"
kind: macro
embedded_support: full
groups: ["Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)"]
related_syntax: ["macro_rules!", "$ident:kind", "$(...)…"]
see_also: ["$ident:kind"]
---

## Explanation

Inside a `macro_rules!` matcher, `$` followed by an identifier introduces
a **metavariable** — a named placeholder that captures whatever tokens
match at that position. `$name:expr` captures a complete expression and
binds it to `$name`; the part after the colon is a **fragment
specifier** that constrains what's legal to capture there — see
[`$ident:kind`](fragment-specifier.md) for the full list of specifiers
and what each one accepts.

In the transcriber (the macro's expansion template), writing `$name`
again substitutes the exact tokens that were captured — not a copy of a
runtime value, but the literal token sequence matched at the call site.
A metavariable can be referenced any number of times in the transcriber,
each reference re-emitting the same captured tokens.

`$` has no meaning in ordinary Rust code outside a macro definition;
`$name` is only ever a metavariable inside a `macro_rules!` matcher or
transcriber. One special metavariable, `$crate`, is always available
without being declared — it expands to a path that always resolves to
the macro's defining crate regardless of where the macro is invoked
from, which matters once a macro is exported (see
[`#[macro_export] / #[macro_use]`](../attributes/macro-export-and-use.md)).

## Basic usage example

```
macro_rules! double {
    ($x:expr) => { $x + $x }; // <- captures $x once, reuses it twice in the expansion
}

let n = double!(21); // <- expands to 21 + 21
```

## Best practices & deeper information

### Scenario: Designing a public API

A calibration helper captures a sensor-reading expression once as
`$raw` and needs it both to build the returned value and to report the
original input. Because a metavariable substitutes raw tokens, not a
shared value, using `$raw` twice would run its expression twice — so the
macro binds it to a local first.

```
macro_rules! calibrate {
    ($raw:expr, $offset:expr) => {{
        let raw = $raw; // <- $raw's tokens are substituted here exactly once
        (raw + $offset, raw) // <- reused as the local `raw`, not by writing $raw again
    }};
}

fn read_sensor() -> f64 {
    println!("reading sensor");
    21.4
}

let (adjusted, original) = calibrate!(read_sensor(), 0.5); // <- read_sensor() runs exactly once
```

**Why this way:** a metavariable re-emits the exact tokens it captured at
every place it's written in the transcriber, so writing `$raw` twice
would re-run `read_sensor()` twice — the
[Rust Reference's macro expansion rules](https://doc.rust-lang.org/reference/macros-by-example.html)
describe substitution as textual, not value-sharing, which is why a
side-effecting or expensive input is bound to a local variable before
being reused.

## Embedded Rust Notes

**Full support.** Metavariable capture and substitution happen entirely
during compilation, before any code is generated, so there's no
`#![no_std]`-specific behavior — the mechanism is identical whether the
expansion targets a hosted or bare-metal build.

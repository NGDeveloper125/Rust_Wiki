---
title: "$ident"
kind: macro
embedded_support: full
groups: ["Macro Definition Syntax", "Macros & Metaprogramming"]
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

## Usage examples

### Reusing a captured metavariable in the expansion

```
macro_rules! double {
    ($x:expr) => { $x + $x }; // <- captures $x once, reuses it twice in the expansion
}

let n = double!(21); // <- expands to 21 + 21
```

### Designing a public API

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

A metavariable re-emits the exact tokens it captured at
every place it's written in the transcriber, so writing `$raw` twice
would re-run `read_sensor()` twice — the
[Rust Reference's macro expansion rules](https://doc.rust-lang.org/reference/macros-by-example.html)
describe substitution as textual, not value-sharing, which is why a
side-effecting or expensive input is bound to a local variable before
being reused.

## Explanation (Embedded)

Metavariable capture and substitution happen entirely during
compilation, before any code is generated, so there's no
`#![no_std]`-specific behavior here — the mechanism is identical whether
the expansion targets a hosted binary or a bare-metal firmware image.
Where this shows up constantly in embedded code is register-definition
and peripheral-boilerplate macros: HAL crates generate dozens of
near-identical register accessors or peripheral wrappers from a handful
of `$name:ident`/`$addr:expr`-style metavariables, rather than
hand-writing the same shape once per register.

## Usage examples (Embedded)

### Capturing a register name and address

```
macro_rules! define_register {
    ($name:ident, $addr:expr) => { // <- captures an identifier and an expression as metavariables
        const $name: usize = $addr; // <- $name and $addr are substituted with the exact captured tokens
    };
}

define_register!(GPIOA_ODR, 0x4800_0014); // <- expands to: const GPIOA_ODR: usize = 0x4800_0014;
```

### Generating a peripheral accessor from captured metavariables

A HAL-style macro captures a register's function name, address, and
value type once, then reuses each metavariable both to name the
generated function and to build its body — the same "capture once,
reference the tokens repeatedly" mechanism the classic examples show,
just applied to register boilerplate instead of arithmetic.

```
macro_rules! ro_register {
    ($fn_name:ident, $addr:expr, $ty:ty) => {
        fn $fn_name() -> $ty { // <- $fn_name names the generated accessor
            unsafe { core::ptr::read_volatile($addr as *const $ty) } // <- $addr and $ty reused in the body
        }
    };
}

ro_register!(read_status_reg, 0x4001_0000, u32);
// expands to: fn read_status_reg() -> u32 { unsafe { core::ptr::read_volatile(0x4001_0000 as *const u32) } }
```

---
title: "macro_rules!"
kind: keyword
embedded_support: full
groups: [Macros, "Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)"]
related_syntax: ["!", "ident!(...) / ident!{...} / ident![...]", "$ident", "$ident:kind", "$(...)…"]
see_also: ["Declarative macros (macro_rules!)"]
---

## Explanation

`macro_rules!` declares a declarative macro: a name, followed by a brace-
delimited list of one or more **arms**, each written `(matcher) =>
{transcriber};` — so a two-armed definition reads as `macro_rules! name {
(matcher_1) => { transcriber_1 }; (matcher_2) => { transcriber_2 }; }`
end to end. A matcher is a sequence of literal tokens and
metavariables (`$name:expr`, `$name:ident`, ...); a transcriber is the
template of tokens to emit when that matcher fits. Arms are separated by
`;`; the `;` after the very last arm is optional.

Invoking `name!(...)` tries each arm's matcher **in the order written,
top to bottom, and uses the first one whose matcher fully consumes the
invocation's tokens** — the same "first fit wins" discipline as `match`,
except the thing being matched is a token tree, not a runtime value. If
no arm matches, the invocation fails to compile with an error naming the
unexpected token. Because matching is purely syntactic and stops at the
first fit, arm order is meaningful: a broad matcher listed before a
narrower, more specific one can swallow input the narrower arm was meant
to handle, so specific literal-token patterns generally belong above
general catch-alls.

By convention (not grammar — see
[`ident!(...) / ident!{...} / ident![...]`](../macros/invocation-forms.md)
for why the delimiters are interchangeable) a matcher is written with
`(...)` and a transcriber with `{...}`, but the compiler accepts any of
the three bracket pairs on either side. A matcher can also contain a
repetition, `$(...)sep*`/`$(...)sep+`/`$(...)?`, so a single arm can
accept a variable-length list of tokens — see
[`$(...)…`](../macros/repetition.md) — and `macro_rules!` macros may
invoke themselves recursively as part of their own expansion, which
combined with repetition is how variadic macros like `vec!` accept any
number of arguments. See
[Declarative macros (macro_rules!)](../../concepts/macros-metaprogramming/declarative-macros.md)
for the broader mental model and when to reach for this mechanism over a
procedural macro.

A reserved but currently unusable keyword, **`macro`**, sits alongside
`macro_rules!` in the language grammar. It's set aside for a possible
future "macro 2.0" declarative-macro syntax intended to fix several of
`macro_rules!`'s well-known ergonomic warts — chiefly its unusual,
match-like invocation grammar and some rough edges in its hygiene and
visibility rules. As of the current stable compiler, `macro` has no
stable syntax or behavior of its own; writing `macro` outside of
experimental/nightly contexts is simply a compile error. Treat it as a
name Rust has set aside for the future, not a usable feature today.

## Usage examples

### Defining a single-arm expression macro

```
macro_rules! double { // <- defines a declarative macro with one arm
    ($x:expr) => { $x + $x };
}

let n = double!(21); // <- expands to 21 + 21
```

### Designing a public API

A helper macro that builds a `SensorReading` needs both a bare shorthand
and an explicit-value form. Because arms are tried top to bottom, the
literal-token arm has to come first, or the general `expr` arm below it
would swallow the shorthand too.

```
struct SensorReading { celsius: f64 }

macro_rules! sensor_reading {
    (default) => { // <- tried first: matches only the bare token `default`
        SensorReading { celsius: 20.0 }
    };
    ($celsius:expr) => { // <- tried second: matches any other expression, including a bare ident
        SensorReading { celsius: $celsius }
    };
}

let a = sensor_reading!(default); // <- matches the first arm
let b = sensor_reading!(18.5);    // <- falls through to the second arm
```

`macro_rules!` arms are matched exactly like `match`
arms — top to bottom, first fit wins — so a specific literal-token
pattern must be listed above a broad `expr` catch-all, or the specific
arm is unreachable dead code; the
[Rust Reference's macro-by-example chapter](https://doc.rust-lang.org/reference/macros-by-example.html)
documents this ordered-matching rule explicitly.

### Testing

A test-helper macro needs both an exact and an approximate comparison
form. The arm containing the literal token `approx` is listed first so
it isn't swallowed by the general two-expression arm below it.

```
macro_rules! assert_reading_eq {
    ($actual:expr, approx $expected:expr) => { // <- tried first: only matches when `approx` literally appears
        assert!(($actual - $expected).abs() < 0.01, "{} != ~{}", $actual, $expected);
    };
    ($actual:expr, $expected:expr) => { // <- tried second: the general two-expression form
        assert_eq!($actual, $expected);
    };
}

#[test]
fn readings_match() {
    assert_reading_eq!(21.0, 21.0);          // <- matches the second, general arm
    assert_reading_eq!(21.003, approx 21.0); // <- matches the first, literal-token arm
}
```

Literal tokens inside a matcher (here, the bare word
`approx`) must appear verbatim in the invocation to match, which lets one
macro name support several distinct call shapes without an `if`/`match`
inside the expansion — the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
conventions still apply to whatever `#[test]` function ultimately runs;
the macro only chooses which assertion shape to generate.

## Embedded Rust Notes

**Full support.** `macro_rules!` expansion happens entirely before code
generation, so it's identical under `#![no_std]` and costs nothing at
runtime. The reserved `macro` keyword has no embedded-specific relevance
either, since it isn't usable syntax on any target yet.

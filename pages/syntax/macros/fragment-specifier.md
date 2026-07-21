---
title: "$ident:kind"
kind: macro
embedded_support: full
groups: ["Macro Definition Syntax", "Macros & Metaprogramming"]
related_concepts: ["Declarative macros (macro_rules!)"]
related_syntax: ["macro_rules!", "$ident", "$(...)…"]
see_also: ["$ident"]
---

## Explanation

The part after the colon in `$name:kind` is the **fragment specifier**:
it tells the matcher exactly what category of syntax `$name` is allowed
to capture at that position. A matcher fails to match (and `macro_rules!`
falls through to the next arm, or fails to compile if none matches) if
the tokens at that position don't parse as the named fragment kind. The
specifiers, and what each captures:

| Specifier | Captures |
|---|---|
| `expr` | a complete expression — `2 + 2`, `sensor.read()`, `if ready { 1 } else { 0 }` |
| `ident` | an identifier, or a keyword used where an identifier is legal (e.g. `self`) |
| `ty` | a type — `u32`, `Vec<String>`, `&str`, `dyn Trait` |
| `pat` | a pattern, as used in `match`/`let` — `Some(x)`, `1..=9`, `_`, including or-patterns |
| `stmt` | a statement without its trailing semicolon — an item, a `let`, or an expression statement |
| `block` | a brace-delimited block expression, `{ ... }`, including its braces |
| `item` | an item — a `fn`, `struct`, `impl`, `mod`, `use`, and so on |
| `literal` | a literal value — `42`, `"text"`, `3.14`, `b'x'` — including a leading `-` on numeric literals |
| `tt` | a single **token tree**: one token, or one bracketed group together with everything inside it, matched with no interpretation at all |
| `path` | a type path — `std::collections::HashMap`, `Self::Item` |
| `meta` | the contents of an attribute — the same grammar accepted inside `#[...]` |
| `vis` | a visibility qualifier, which may be empty — `pub`, `pub(crate)`, or nothing at all |
| `lifetime` | a lifetime or loop label — `'a`, `'static`, `'outer` |

`tt` is the least constrained specifier: it doesn't try to parse its
input as any particular kind of Rust syntax, so it matches almost
anything one token (or one delimited group) at a time. The tradeoff runs
the other way for every other specifier: once a fragment is captured as
`expr`, `ty`, `pat`, and so on, it becomes an **opaque** unit for every
rule that follows — a later arm can't pattern-match *inside* a captured
`expr` to see, say, whether it contains a particular operator. `tt` (often
paired with [repetition](repetition.md) to walk over several of them) is
the escape hatch when a macro genuinely needs to see the raw tokens
rather than have them parsed away.

## Usage examples

### Matching an expression fragment

```
macro_rules! print_typed {
    ($value:expr) => { println!("{}", $value) }; // <- `expr` accepts any complete expression
}

print_typed!(2 + 2); // <- prints 4
```

### Designing a public API

An assertion macro needs to see which comparison operator was written so
it can pick the matching `assert!` message — but `expr` would swallow
the whole comparison as one opaque unit, leaving the operator invisible
to the macro. `tt` keeps each side, and the operator between them,
separately visible.

```
macro_rules! assert_op {
    ($lhs:tt > $rhs:tt) => { // <- `tt` captures each side separately, keeping `>` visible to match on
        assert!($lhs > $rhs, "{} is not greater than {}", stringify!($lhs), stringify!($rhs));
    };
    ($lhs:tt < $rhs:tt) => {
        assert!($lhs < $rhs, "{} is not less than {}", stringify!($lhs), stringify!($rhs));
    };
}

let pressure = 120;
let threshold = 100;

// AVOID: a single `$check:expr` would swallow `pressure > threshold` as one opaque unit,
// leaving the macro no way to see or branch on which operator was used.

assert_op!(pressure > threshold); // PREFER: `tt` keeps `>` visible so the matcher can pick the right arm
```

Once a fragment matches as `expr`, it can't be
inspected or re-matched by a later rule — the
[Rust Reference's fragment-specifier semantics](https://doc.rust-lang.org/reference/macros-by-example.html#metavariables)
document `expr` as producing an opaque AST node, which is exactly why
`tt` is chosen deliberately whenever a macro's own logic depends on
seeing a specific token, like an operator, rather than having it parsed
away.

## Explanation (Embedded)

Fragment specifiers are resolved entirely at macro-expansion time, before
any code generation happens — there's no runtime representation at all,
so every specifier in the table above behaves identically whether the
macro's expansion ends up compiled for a hosted target or `#![no_std]`
firmware. There's no embedded-specific nuance to the matching rules
themselves.

Where this genuinely shows up is *what kind* of macro gets written this
way in the first place: register- and peripheral-definition boilerplate
is one of the most common reasons to hand-write a `macro_rules!` in
embedded Rust, and those macros lean on `ident` (to name the generated
register/field/function) and `tt`/`literal` (to carry a bit position or
mask) far more than on `expr`, since the macro is generating code that
*names hardware*, not evaluating a runtime value.

## Usage examples (Embedded)

### Generating a GPIO register accessor from raw identifiers

```
macro_rules! gpio_pin {
    ($name:ident, $offset:literal) => { // <- `ident` names the generated fn, `literal` carries the bit position
        pub fn $name(odr: &mut u32) {
            *odr |= 1 << $offset;
        }
    };
}

gpio_pin!(set_led, 5); // <- expands to `pub fn set_led(odr: &mut u32) { *odr |= 1 << 5; }`
```

### Matching an interrupt source with a pat fragment

```
macro_rules! on_interrupt {
    ($source:pat => $body:block) => { // <- `pat` accepts the enum variant naming the interrupt source
        match interrupt::pending() {
            $source => $body,
            _ => {}
        }
    };
}

on_interrupt!(Interrupt::Uart0 => {
    handle_uart_rx();
});
```

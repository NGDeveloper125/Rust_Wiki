---
title: ";"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: ["{ }"]
see_also: ["{ }"]
---

## Explanation

`;` terminates a statement, and — just as importantly in an
expression-oriented language — turns what would otherwise be an
expression into a statement that evaluates to `()`:

```
let x = 5;      // statement
x + 1;          // expression, discarded, statement evaluates to ()
x + 1           // expression, this IS the value (no semicolon)
```

This is why leaving off the trailing semicolon on a block's last line
means "return this value," while adding one means "run this and discard
the result." Getting this wrong (an accidental trailing `;`) is one of the
most common beginner mistakes — a function meant to return a value
silently returns `()` instead, usually caught only by a type error at the
call site.

`;` also appears inside `[Type; N]` (array type/literal — see
[square brackets](square-brackets.md)), which is unrelated to its
statement-terminator role.

## Basic usage example

```
fn increment(x: i32) -> i32 {
    x + 1 // <- no `;`: this expression IS the function's return value
}

fn increment_wrong(x: i32) -> i32 {
    x + 1; // <- `;` turns this into a statement; the block now returns `()`
}
```

**Restriction:** `increment_wrong` above fails to compile — its body
evaluates to `()`, which doesn't match the declared `-> i32` return type.

## Best practices & deeper information

### Scenario: Handling and propagating errors

An early-return guard clause needs its own `;`, while the function's
final, successful value must not have one — mixing this up inside a
`?`-chain is exactly the "silently returns `()`" trap the Explanation
above describes, and it's easy to hit in a longer function.

```
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path)?; // <- `;` ends this statement
    if text.is_empty() {
        return Err(ConfigError::Empty); // <- `;` ends this early-return statement
    }
    // AVOID: a trailing `;` here would make the fn return () instead of Result<Config, _>
    parse_config(&text) // PREFER: no `;` — this expression IS the fn's return value
}
```

**Why this way:** `clippy` specifically watches for this class of mistake
via lints like
[`unit_arg`](https://rust-lang.github.io/rust-clippy/master/#unit_arg) —
but the more reliable habit is to read a function's last line and ask
"is this a statement or the return value?" before deciding whether it
needs a `;`.

### Scenario: Working with collections

Inside `[Type; N]` and `[value; N]`, `;` separates a type/value from a
compile-time length — a completely different grammatical role from
statement-termination, disambiguated purely by appearing inside `[ ]`.

```
let buffer: [u8; 4] = [0; 4]; // <- both `;` here belong to array syntax, not statements
//           ^ type; length      ^ value; repeat count
```

**Why this way:** `[0; 4]` avoids writing out `[0, 0, 0, 0]` by hand and,
unlike a `Vec`, the length is part of the type itself — see
[Arrays vs `Vec`](../../concepts/types-data-modeling/arrays-vs-vec.md) for
when a fixed-size array is the right choice over a growable one.

## Embedded Rust Notes

**Full support.** Pure statement-terminator grammar — no `std` dependency.

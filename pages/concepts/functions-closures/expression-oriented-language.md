---
title: "Expression-oriented language"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures", "Functional Programming"]
related_syntax: [";", "{ }", if, else, loop, match]
see_also: ["Functions", "Closures & capturing"]
---

## Explanation

In Rust, almost everything evaluates to a value: `if`/`else`, `match`,
`loop`, and even a plain `{ }` block are expressions, not statements. This
is a different default from C-family languages, where `if` only decides
which branch of code *runs* and never produces a value itself. In Rust,
`if condition { a } else { b }` *is* a value — usable directly as the
right-hand side of a `let`, a function argument, or the tail of another
block.

This matters because it removes an entire category of boilerplate: rather
than declaring a variable, then mutating it inside each branch of an
`if`/`else` or `match`, you write the branches so each one produces the
final value directly, and bind it once. The same idea applies to
[function](functions.md) bodies — a function's final expression, with no
trailing semicolon, is its return value, which is why idiomatic Rust
functions rarely need an explicit `return`.

The semicolon is what separates the two: appending `;` turns an
expression into a statement, discarding whatever value it produced (an
expression-statement's value becomes `()`, unit). This is also why adding
or removing a trailing semicolon on the last line of a function body
changes what the function returns — a common early surprise, and a direct
consequence of the expression/statement distinction rather than an
arbitrary rule.

Closures follow the same rule as functions: a closure's body is an
expression too, which is why a short closure can be written on one line
without braces at all — see [Closures & capturing](closures-and-capturing.md).
Being expression-oriented is also one of the traits Rust shares with
functional languages, alongside pattern matching and immutability by
default, even though Rust itself is not a purely functional language.

## Basic usage example

```
let score = 82;
let grade = if score >= 90 { "A" } else if score >= 80 { "B" } else { "C" };
// <- `if`/`else` is an expression here: it produces `grade` directly, no `mut` needed
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

Computing a shipping fee from an order's weight tier is a direct value
computation, so `match` is written as an expression that produces the fee,
rather than a `mut` variable assigned inside each arm.

```
enum WeightTier {
    Light,
    Standard,
    Heavy,
}

fn shipping_fee_cents(tier: &WeightTier) -> u32 {
    match tier { // <- the whole `match` evaluates to a value; the function just returns it
        WeightTier::Light => 300,
        WeightTier::Standard => 700,
        WeightTier::Heavy => 1500,
    }
}
```

**Why this way:** every arm must produce a value of the same type, which
the [Rust Reference](https://doc.rust-lang.org/reference/expressions/match-expr.html)
documents as part of `match`'s grammar as an expression — the compiler
rejects a match where the arms' value types disagree, catching the
mistake before it ships.

### Scenario: Validating input

Validating a raw string into a plausible age produces either an `Ok` or an
`Err` value; writing that as an `if`/`else` expression keeps the result
value's construction next to the condition that decides it.

```
fn validate_age(raw: &str) -> Result<u8, String> {
    let age: u8 = raw.trim().parse().map_err(|_| "not a number".to_string())?;

    if age == 0 || age > 130 { // <- `if` as an expression: it produces the Err/Ok value directly
        Err(format!("{age} is not a plausible age"))
    } else {
        Ok(age)
    }
}
```

**Why this way:** letting the `if`/`else` produce the `Result` directly
avoids a separate `mut` "result" variable reassigned in each branch — the
kind of unnecessary mutability the
[Rust Design Patterns' temporary-mutability idiom](https://rust-unofficial.github.io/patterns/idioms/temporary-mutability.html)
recommends avoiding in favor of scoping the computation to an expression.

### Scenario: Creating a new object

Building a `Config` where one field depends on a flag reads more clearly
when that field is computed by an `if` expression bound once, rather than
declared `mut` and reassigned before the struct literal.

```
struct Config {
    retries: u32,
    timeout_ms: u32,
}

fn build_config(verbose: bool) -> Config {
    let timeout_ms = if verbose { 10_000 } else { 3_000 };
    // <- expression assigns the final value directly; `timeout_ms` is never `mut`

    Config {
        retries: 3,
        timeout_ms,
    }
}
```

**Why this way:** binding the result of an expression once, instead of
mutating a placeholder, keeps every binding's lifetime as short and as
immutable as possible — a pattern the
[Rust Design Patterns book](https://rust-unofficial.github.io/patterns/idioms/temporary-mutability.html)
calls out directly as an idiom to prefer over temporary mutability.

## Embedded Rust Notes

**Full support.** Being expression-oriented is a compile-time property of
the language's grammar — `if`/`match`/block expressions compile to the
same branching code on a microcontroller as they would on a server, with
no dependency on `std`, an allocator, or an OS.

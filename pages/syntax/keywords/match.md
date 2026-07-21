---
title: "match"
kind: keyword
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching"]
related_concepts: ["match expressions", "Match guards", "Exhaustiveness checking", "Destructuring"]
related_syntax: ["=>", "|", "@", "if let", "_"]
see_also: ["=>", "if let", "@"]
---

## Explanation

`match` compares a scrutinee value against a sequence of arms, each written
`pattern => expression`, and runs the first arm whose pattern fits. Arms are
separated by commas; the comma after an arm is optional when the arm's
expression is itself a `{ ... }` block, since the closing brace already ends
the arm. See [match expressions](../../concepts/pattern-matching/match-expressions.md)
for the deeper "why" — when `match` earns its keep over lighter forms like
`if let`, and how it relates to designing with enums.

Several patterns can share one arm by joining them with
[`|`](../operators/pipe.md): `Some(1) | Some(2) => ...` runs the same
expression for either shape. The full grammar of or-patterns lives on that
page and isn't repeated here.

An arm's pattern can carry an extra condition, called a match guard, written
`pattern if condition => expression`. The guard runs only after the pattern
has already matched, and can reference any name the pattern just bound —
grammar-wise, it's an arbitrary boolean expression tacked onto an otherwise
ordinary arm. See [match guards](../../concepts/pattern-matching/match-guards.md)
for when reaching for one is the right call versus reshaping the pattern
instead.

`identifier @ pattern` binds `identifier` to the value being matched while
still requiring that value to match `pattern` — see [`@`](../punctuation/at-sign.md)
for the full grammar of binding patterns; `match` arms are the most common
place they appear.

`match` requires its arms to be **exhaustive**: every value the scrutinee's
type could hold must be covered by some arm's pattern, or the code fails to
compile. This check only looks at patterns, not guards — an arm with a guard
doesn't count as covering its pattern for exhaustiveness purposes, since the
guard might evaluate to `false` at runtime and the compiler can't rule that
out. A trailing `_ => expression` arm (see [`_`](../punctuation/underscore.md))
is the usual way to satisfy exhaustiveness when the remaining cases don't
need distinct handling. See
[exhaustiveness checking](../../concepts/pattern-matching/exhaustiveness-checking.md)
for what this guarantees and why it matters beyond the grammar rule itself.

Like `if`, `match` is an expression: when every arm's expression has the same
type and none end with a semicolon, the whole `match` evaluates to whichever
arm ran, and can sit on the right-hand side of a `let` or as a function's
final expression.

## Basic usage example

```
enum Direction { North, South, East, West }

let heading = Direction::East;

let degrees = match heading { // <- `match` picks the arm whose pattern fits
    Direction::North => 0,
    Direction::South => 180,
    Direction::East => 90,
    Direction::West => 270,
};

println!("{degrees}");
```

**Restriction:** every possible value of the scrutinee's type must be
covered by some arm's pattern, or this fails to compile — see
[exhaustiveness checking](../../concepts/pattern-matching/exhaustiveness-checking.md).

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

A thermostat's incoming command needs to both identify its shape and, for
the temperature-setting command, confirm the requested value is in a
sane range while still keeping the exact number to report back.

```
enum ThermostatCommand {
    SetTemperature(i32),
    Boost,
    Off,
}

fn describe(cmd: &ThermostatCommand) -> String {
    match cmd { // <- `match` on the command enum
        ThermostatCommand::SetTemperature(t @ 16..=30) => {
            // <- `@` binds `t` to the value while also checking the range
            format!("setting target to {t}C")
        }
        ThermostatCommand::SetTemperature(t) => format!("ignoring out-of-range target {t}C"),
        ThermostatCommand::Boost | ThermostatCommand::Off => "no target change".to_string(),
        // <- `|` lets one arm handle two variants that need identical handling
    }
}
```

**Why this way:** combining `|` for variants that share behavior and `@`
for variants that need both a shape check and the underlying value keeps
the dispatch in one `match` rather than a chain of separate `if`s — the
[Rust Reference on match expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html)
documents both as first-class parts of an arm's pattern, not add-on sugar.

### Scenario: Handling and propagating errors

Classifying a sensor reading string needs different messages for a
parse failure versus a suspiciously cold (but valid) reading — a guard
distinguishes the second case from the general success case.

```
fn classify_reading(raw: &str) -> String {
    match raw.trim().parse::<f64>() {
        Ok(temp) if temp < 0.0 => format!("reading {temp} is below freezing"),
        // <- guard: same `Ok(temp)` shape, extra condition on the value
        Ok(temp) => format!("reading {temp} is nominal"),
        Err(e) => format!("invalid reading {raw:?}: {e}"),
    }
}

println!("{}", classify_reading("-4.5"));
```

**Why this way:** matching on the `Result` directly, with a guard for the
one `Ok` sub-case that needs different handling, avoids re-parsing or a
second `if` after the match — the
[Rust Book](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#extra-conditionals-with-match-guards)
covers guards as exactly this kind of value-dependent refinement on an
already-matched pattern.

### Scenario: Validating input

Routing a delivery by distance needs every possible `u32` value handled,
not just the bands anyone thought of in advance — the exhaustiveness
requirement is what forces a catch-all here.

```
fn shipping_zone(distance_km: u32) -> &'static str {
    match distance_km { // <- exhaustive: distance_km could be any u32, so every value needs a home
        0..=10 => "local",
        11..=100 => "regional",
        101..=1000 => "national",
        _ => "international", // <- required: no fixed range of literals ever covers all of u32
    }
}
```

**Why this way:** because `0..=1000` doesn't come close to covering every
`u32`, the compiler refuses to compile this without a final wildcard —
the same exhaustiveness rule that requires every enum variant to appear
also requires numeric patterns to visibly account for every value in
range, per the
[Rust Reference on exhaustiveness](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-expressions).

## Embedded Rust Notes

**Full support.** `match` is core-language, allocator-free, and compiles to
a jump table or comparison chain with no runtime cost beyond that — a
standard way to decode a protocol byte or peripheral status register into
one of several known shapes, with the compiler guaranteeing every expected
bit pattern is handled.

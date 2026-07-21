---
title: "@"
kind: punctuation
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching"]
related_concepts: ["Destructuring", "match expressions"]
related_syntax: [match, ".. / ..= / ...", "|"]
see_also: [match, ".. / ..= / ..."]
---

## Explanation

`ident @ pattern` binds `ident` to the entire value being matched, while
still requiring that value to match `pattern`. It appears in any pattern
position — most often a [`match`](../keywords/match.md) arm, but also
`if let`/`while let` and `let` patterns.

`@` exists because, without it, a pattern position forces a choice between
two things that are each easy alone but awkward together:

- Bind a name to the matched value: `n => ...` gets the whole value, but
  the pattern itself says nothing narrower about its shape than the type
  already does.
- Match a specific sub-pattern, such as a literal or range: `1..=5 => ...`
  confirms the value falls in that range, but the arm has no name to refer
  to the exact value that matched — only the range's boundaries are known
  statically from the pattern text.

`ident @ pattern` gets both in one step: the arm's body can use `ident` as
the exact matched value, having already confirmed it fits `pattern`.

`@` composes with other pattern forms. With an or-pattern (see
[`|`](../operators/pipe.md)), wrapping the alternatives in parentheses lets
one binding cover all of them: `n @ (1 | 2 | 3)`. With a struct or
enum-variant pattern, `@` can bind one field's value while the rest of the
pattern still [destructures](../../concepts/pattern-matching/destructuring.md)
the surrounding shape, e.g. `Reading { value: v @ 0.0..=100.0, .. }`.

## Usage examples

### Binding a name while matching a range pattern

```
let reading = 42;

match reading {
    n @ 1..=50 => println!("band A: {n}"), // <- `@` binds `n` to the value while checking the range
    n @ 51..=100 => println!("band B: {n}"),
    other => println!("out of range: {other}"),
}
```

### Branching on data (pattern matching)

Classifying an engine's RPM into a warning band still needs the exact
reading for the log message — `@` keeps both the classification and the
number in one arm.

```
enum EngineState {
    Idle,
    Nominal,
    Warning,
}

fn classify(rpm: u32) -> (EngineState, String) {
    match rpm {
        n @ 0..=800 => (EngineState::Idle, format!("{n} rpm, idling")),
        n @ 801..=5000 => (EngineState::Nominal, format!("{n} rpm, nominal")),
        n @ 5001..=7000 => {
            // <- `@` keeps the exact rpm for the message while matching the redline band
            (EngineState::Warning, format!("{n} rpm, approaching redline"))
        }
        n => (EngineState::Warning, format!("{n} rpm, over redline")),
    }
}

println!("{}", classify(6200).1);
```

Without `@`, reporting the exact rpm in the warning
message would need a second lookup or a re-derivation of the value the
pattern already had in hand — the
[Rust Reference on identifier patterns](https://doc.rust-lang.org/reference/patterns.html#identifier-patterns)
documents `@` as exactly this: binding a name to a value a sub-pattern has
already matched.

### Validating input

Rejecting an out-of-range port still needs to say which value was invalid
— `@` retains the checked value so it can flow straight into the `Ok`
result without a second read.

```
fn validate_port(candidate: i32) -> Result<u16, String> {
    match candidate {
        p @ 1..=65535 => Ok(p as u16), // <- `@` binds the checked value so it can be returned directly
        bad => Err(format!("port {bad} is out of range")),
    }
}
```

The alternative — matching `1..=65535` with no binding,
then re-reading `candidate` inside the arm — works but discards the
guarantee the pattern already established; binding with `@` makes the
"this value, already checked" relationship explicit at the type level, not
just by convention.

## Explanation (Embedded)

`@` means exactly the same thing under `#![no_std]` — pure pattern-matching
grammar, resolved at compile time with no allocation and no runtime cost.
It's a natural fit for a driver's status type: a "reading" enum often
wraps a raw sensor or register value inside a variant, and `@` lets one
match arm both classify that variant *and* keep the raw value on hand for
a log line or telemetry frame, without reading the peripheral a second
time.

## Usage examples (Embedded)

### Binding a field while matching a hardware status enum

```
use defmt::info;

enum SensorReading {
    Temperature(i16),
}

fn classify(reading: SensorReading) {
    match reading {
        SensorReading::Temperature(t @ -40..=85) => info!("temp {} C: in spec", t), // <- `@` binds `t` while checking the range
        SensorReading::Temperature(t) => info!("temp {} C: OUT OF SPEC", t),
    }
}
```

### Keeping a raw ADC code while classifying it into a band

```
fn classify_adc(raw: u16) {
    match raw {
        v @ 0..=819 => defmt::info!("adc {}: low band", v), // <- `@` keeps the raw code for the log line
        v @ 820..=3276 => defmt::info!("adc {}: mid band", v),
        v => defmt::info!("adc {}: high band", v),
    }
}
```

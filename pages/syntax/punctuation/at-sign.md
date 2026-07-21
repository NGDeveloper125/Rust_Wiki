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

## Embedded Rust Notes

**Full support.** Pure pattern-matching grammar, allocator-free. A natural
fit for classifying an ADC reading or register value into a named band
while keeping the raw value on hand for logging or telemetry, in one
`match` arm instead of a lookup followed by a separate range check.

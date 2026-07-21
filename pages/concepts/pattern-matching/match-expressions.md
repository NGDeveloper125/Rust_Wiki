---
title: "match expressions"
area: "Pattern Matching"
embedded_support: full
groups: ["Pattern Matching", "Functional Programming", "Designing Robust Data Models", "Coming from Haskell / functional languages"]
related_syntax: [match, "=>", "|", "_"]
see_also: ["if let / while let", "Destructuring", "Match guards", "Exhaustiveness checking", "Enums (algebraic data types)", "Option<T>", "Result<T, E>"]
---

## Explanation

A `match` expression compares a value against a series of patterns and
runs the code for whichever pattern fits, and — unlike a chain of `if`
statements — the compiler checks that every possible shape of the value
is covered before it will compile. It is the central tool for branching
on *structure* rather than on a boolean condition: instead of asking
"is this true?" repeatedly, `match` asks "which of these shapes is it?"
once, and the answer determines both the code that runs and, often, the
data that gets extracted along the way.

Calling it a `match` *expression*, not a `match` statement, is
deliberate: every arm produces a value of the same type, and the whole
`match` evaluates to whichever arm ran, so it can sit directly on the
right-hand side of a `let` or as a function's final return value. This
follows naturally from Rust being an expression-oriented language — `if`
works the same way — and it means branching logic that picks a value
doesn't need a mutable placeholder variable assigned from inside each
branch.

`match` is most powerful against [enums](../types-data-modeling/enums-algebraic-data-types.md),
because each arm can both identify which variant a value is *and*
[destructure](destructuring.md) the data that variant carries in the
same step — there is no separate "check the tag, then reach in and pull
the field out" dance. The compiler's insistence on covering every
variant is [exhaustiveness checking](exhaustiveness-checking.md), and
it is what turns "I forgot a case" from a runtime bug into a compile
error, which is a large part of why `match` feels different from a
`switch` in C-like languages.

For simpler cases — only one pattern actually matters, and everything
else should be ignored or handled identically — reaching for the
lighter [`if let` / `while let`](if-let-and-while-let.md) forms avoids
the ceremony of writing out arms nobody cares about. `match` earns its
keep when several variants each need distinct handling, or when
[match guards](match-guards.md) need to add a condition on top of the
shape being matched.

## Basic usage example

```
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

let light = TrafficLight::Yellow;

let instruction = match light { // <- a match expression: the whole thing evaluates to a value
    TrafficLight::Red => "stop",
    TrafficLight::Yellow => "slow down",
    TrafficLight::Green => "go",
};

println!("{instruction}");
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

A network client's session moves through a handful of distinct states,
and a status line needs to describe whichever one it's currently in —
exactly the kind of branching-on-structure `match` is built for.

```
enum SessionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Closing { reason: String },
}

fn status_line(state: &SessionState) -> String {
    match state { // <- exhaustive: every SessionState variant must appear below
        SessionState::Disconnected => "disconnected".to_string(),
        SessionState::Connecting { attempt } => format!("connecting (attempt {attempt})"),
        SessionState::Connected { session_id } => format!("connected as {session_id}"),
        SessionState::Closing { reason } => format!("closing: {reason}"),
    }
}
```

**Why this way:** matching directly on the enum, rather than adding a
separate `is_connected()`-style method per state, keeps the state and
its data next to the code that reacts to it — the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html) frames
`match` as the idiomatic way to run different code depending on which
variant a value holds.

### Scenario: Handling and propagating errors

Parsing a configured port number can fail, and the caller needs to react
differently to "not a number" than to any other problem — `match` on the
`Result` makes both outcomes explicit instead of unwrapping and hoping.

```
fn parse_port(raw: &str) -> u16 {
    match raw.parse::<u16>() { // <- matches on the Result's Ok/Err shape directly
        Ok(port) => port,
        Err(_) => {
            eprintln!("invalid port {raw:?}, falling back to 8080");
            8080
        }
    }
}

let port = parse_port("not-a-number");
```

**Why this way:** matching `Ok`/`Err` explicitly at the point where a
sensible fallback exists is clearer than propagating with `?` and
handling it far away — the
[Rust Book](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
uses this same `match`-on-`Result` shape for exactly this "recover with
a default" case.

### Scenario: Designing a public API

A vending machine's behavior differs by state, and modeling that as an
enum with a single `match`-based transition function keeps every legal
transition in one place instead of scattered across boolean flags.

```
enum MachineState {
    Idle,
    AwaitingSelection { credit_cents: u32 },
    Dispensing { item: String },
}

enum Event {
    CoinInserted(u32),
    ItemSelected(String),
    DispenseComplete,
}

fn transition(state: MachineState, event: Event) -> MachineState {
    match (state, event) { // <- matching a tuple dispatches on state and event together
        (MachineState::Idle, Event::CoinInserted(cents)) =>
            MachineState::AwaitingSelection { credit_cents: cents },
        (MachineState::AwaitingSelection { credit_cents }, Event::ItemSelected(item)) =>
            MachineState::Dispensing { item },
        (MachineState::Dispensing { .. }, Event::DispenseComplete) => MachineState::Idle,
        (other, _) => other, // <- ignores events that don't apply to the current state
    }
}
```

**Why this way:** encoding the state machine as an enum plus one
exhaustive `match` makes illegal transitions (dispensing without a
selection, say) impossible to reach by construction, which is the
state-machine-via-enum idiom described in
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/).

## Embedded Rust Notes

**Full support.** `match` is core-language, allocator-free, and compiles
down to a jump table or a chain of comparisons with no runtime cost
beyond that. It is the standard way to dispatch on a peripheral's status
register or decode a protocol byte into one of several message kinds,
with exhaustiveness checking guaranteeing every bit pattern the code
expects is actually handled.

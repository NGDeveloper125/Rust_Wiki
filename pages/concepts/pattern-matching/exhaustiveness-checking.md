---
title: "Exhaustiveness checking"
area: "Pattern Matching"
embedded_support: full
groups: ["Pattern Matching", "Unique to Rust"]
related_syntax: [match, "_", "#[non_exhaustive]"]
see_also: ["match expressions", "Enums (algebraic data types)", "Match guards"]
---

## Explanation

Exhaustiveness checking is the compiler proving, before it will produce
a binary, that a [`match`](match-expressions.md) accounts for every
value its scrutinee could possibly hold. Match a `bool` and both `true`
and `false` must appear (or a wildcard covering whatever's left); match
an [enum](../types-data-modeling/enums-algebraic-data-types.md) and
every variant must be handled. Leave one out, and the code simply
doesn't compile — the missing case is a compile error naming exactly
which pattern is unhandled, not a bug waiting to be discovered when that
case finally occurs in production.

This is a meaningfully different guarantee from a `switch` in C-like
languages, where an unhandled case silently falls through (or falls to
a `default` that may be wrong), and from checking with `if`/`else if`,
where a missing case just means "nothing happens" without the compiler
ever registering an omission. Rust's approach turns "did I handle
everything" from a discipline the programmer maintains by memory into a
fact the compiler verifies mechanically, every time, on every `match` in
the codebase — a large part of why it's counted among the
language's genuinely distinctive guarantees rather than a stylistic
nicety.

The practical payoff shows up over time, not just on day one: adding a
new variant to an enum later turns every `match` elsewhere in the
codebase that needs updating into a compile error at the exact line
that needs attention, instead of a silent gap in behavior that only
surfaces when that new variant actually occurs at runtime. This is the
same property that makes
["make invalid states unrepresentable"](../types-data-modeling/enums-algebraic-data-types.md)
work as a design strategy — the enum defines the complete set of valid
shapes, and exhaustiveness checking is what forces every consumer to
keep up as that set changes.

That guarantee is strong enough that library authors sometimes need an
escape hatch: `#[non_exhaustive]` on a public enum tells downstream
crates that more variants may be added later without it being a breaking
change, which in turn forces every external `match` on that enum to
include a wildcard arm — trading a little exhaustiveness at the library
boundary for the freedom to grow the enum without breaking every
consumer on every release.

## Basic usage example

```
enum LogLevel {
    Info,
    Warning,
    Error,
}

fn label(level: &LogLevel) -> &'static str {
    match level { // <- every LogLevel variant must appear, or this fails to compile
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
        // omitting any arm here (e.g. LogLevel::Error) is a compile error, not a runtime gap
    }
}

println!("{}", label(&LogLevel::Warning));
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

A sensor's status is modeled as an enum specifically so that adding a
new failure mode later is impossible to overlook anywhere the status is
handled.

```
enum SensorStatus {
    Ok(f64),
    Miscalibrated,
    Offline,
}

fn alert_level(status: &SensorStatus) -> u8 {
    match status { // <- exhaustive: adding a variant here forces every caller to update
        SensorStatus::Ok(_) => 0,
        SensorStatus::Miscalibrated => 1,
        SensorStatus::Offline => 2,
    }
}

println!("{}", alert_level(&SensorStatus::Offline));
```

**Why this way:** if `SensorStatus` later gains a `Jammed` variant, every
`match` like this one fails to compile until it's handled — the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html#matches-are-exhaustive)
calls out exhaustiveness as the guarantee that makes this kind of
enum-driven design safe to extend over time.

### Scenario: Designing a public API

A payment library's `PaymentMethod` enum is a natural candidate for
future growth (new payment providers get added constantly), so it's
marked `#[non_exhaustive]` — downstream code must include a wildcard
arm, trading a little exhaustiveness for the ability to add variants
without a breaking release.

```
#[non_exhaustive] // <- tells downstream crates more variants may be added later without it being breaking
pub enum PaymentMethod {
    Card,
    BankTransfer,
}

// in a downstream crate consuming this library:
fn describe(method: &PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::Card => "card",
        PaymentMethod::BankTransfer => "bank transfer",
        _ => "unsupported payment method", // <- required by #[non_exhaustive]; handles future variants
    }
}
```

**Why this way:** without `#[non_exhaustive]`, adding a variant to a
published enum is a breaking change for every downstream exhaustive
`match`; the
[API Guidelines' section on future-proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends `#[non_exhaustive]` specifically for enums (and structs) a
library expects to grow, so it can extend them without a semver-major
bump.

## Embedded Rust Notes

**Full support.** Exhaustiveness checking is a compile-time-only
guarantee with zero runtime cost and no allocator dependency. It's
particularly valuable when decoding a fixed set of hardware states or
protocol message kinds from raw register bits — the compiler guarantees
every defined state is handled, which matters more, not less, in code
that can't easily be patched after it ships.

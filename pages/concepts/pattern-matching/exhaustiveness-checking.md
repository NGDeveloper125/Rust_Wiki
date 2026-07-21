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

## Explanation (Embedded)

Exhaustiveness checking is a compile-time-only guarantee: zero runtime
cost, no allocator dependency, identical proof obligation under
`#![no_std]` as anywhere else. What changes is how much that guarantee
is worth. Hosted code that forgets a case usually still fails loudly —
an unhandled `enum` variant that slips past `match` would show up as a
crash with a backtrace, filed as a bug, seen by a user. A microcontroller
decoding a hardware status register or a peripheral's discrete power-mode
enum has none of that safety net: no user watching the console, often no
crash reporting at all, sometimes not even a way to reboot itself
cleanly. An unhandled hardware state there doesn't announce itself — it's
a silent hang, a motor left running, or state that quietly drifts wrong
until someone notices the physical symptom. Exhaustiveness checking
turns that entire failure mode into a compile error, before the firmware
image is ever flashed to a device that may run unattended for years.

## Basic usage example (Embedded)

```
enum PowerMode {
    Active,
    Idle,
    Standby,
    Shutdown,
}

fn duty_cycle_percent(mode: &PowerMode) -> u8 {
    match mode { // <- every PowerMode variant must appear, or this fails to compile
        PowerMode::Active => 100,
        PowerMode::Idle => 40,
        PowerMode::Standby => 5,
        PowerMode::Shutdown => 0,
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Branching on data (pattern matching)

A motor controller's fault register decodes into a fixed set of fault
codes; matching exhaustively guarantees the firmware image flashed to a
fleet of unattended devices can't ship with a fault code silently
ignored — there's no crash report to surface the gap later.

```
enum MotorFault {
    OverCurrent,
    OverTemperature,
    StallDetected,
    EncoderFault,
}

fn shutdown_required(fault: &MotorFault) -> bool {
    match fault { // <- exhaustive: a new fault code added later fails every match like this until handled
        MotorFault::OverCurrent => true,
        MotorFault::OverTemperature => true,
        MotorFault::StallDetected => false,
        MotorFault::EncoderFault => true,
    }
}
```

**Why this way:** on a device nobody is watching, an unhandled fault
code isn't a visible crash to file a bug against — it's a motor that
keeps running, or a controller that silently hangs, with no diagnostic
trail at all; forcing the match to name every defined `MotorFault`
variant stands in for the bug report a hosted program would eventually
get, per the same exhaustiveness guarantee the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html#matches-are-exhaustive)
documents.

### Scenario: Designing a public API

A hardware-abstraction-layer crate's `ChipStatus` enum models every
status bit the current silicon revision defines; marking it
`#[non_exhaustive]` lets a later chip revision add a status the HAL
didn't anticipate without breaking every firmware project's exhaustive
match against the old release.

```
#[non_exhaustive] // <- a future silicon revision may define new status bits this HAL release doesn't know
pub enum ChipStatus {
    Ready,
    Calibrating,
    ThermalThrottle,
}

// in firmware built against this HAL:
fn led_pattern(status: &ChipStatus) -> u8 {
    match status {
        ChipStatus::Ready => 0b0001,
        ChipStatus::Calibrating => 0b0010,
        ChipStatus::ThermalThrottle => 0b0100,
        _ => 0b1111, // <- required by #[non_exhaustive]; a status this HAL version doesn't recognize yet
    }
}
```

**Why this way:** without `#[non_exhaustive]`, a HAL adding a status bit
for a newer chip stepping would break every downstream firmware's
exhaustive match on a minor version bump; the
[API Guidelines' section on future-proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends it for exactly this kind of enum a crate expects to grow —
doubly so for hardware-tracking enums, where new silicon revisions are a
certainty rather than a maybe.

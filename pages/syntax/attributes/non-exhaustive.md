---
title: "#[non_exhaustive]"
kind: attribute
embedded_support: full
groups: ["Types & Layout", "Types & Data Structures", "Pattern Matching"]
related_concepts: ["Exhaustiveness checking", Structs, "Enums (algebraic data types)"]
related_syntax: [struct, enum, match, "_"]
see_also: ["Exhaustiveness checking"]
---

## Explanation

`#[non_exhaustive]` is placed directly above a `struct` item, an `enum`
item, or an individual variant inside an `enum`, and takes no arguments.
Its exact effect depends on which of those three it's attached to, and
in every case the effect applies only to code **outside** the crate that
defines the item — inside the defining crate, the attribute is inert.

On a **struct**, it removes the ability to build the type with a plain
struct-literal (`StructName { field: value, ... }`) from another crate,
even when every field is already `pub` — downstream code must go through
whatever constructor function or method the defining crate provides
instead. It also forces any destructuring pattern on that struct from
outside the crate to include a `..` rest pattern, even one that already
names every field currently defined, since the crate reserves the right
to add fields later without that counting as a breaking change.

On an **enum**, it forces every downstream `match` on that enum to
include a wildcard arm (`_ => ...`) or an equivalent catch-all binding —
the compiler will not treat an external match as exhaustive without one,
because the crate may add variants in a later, semver-compatible release.
Matches written inside the defining crate itself are unaffected and can
still be fully exhaustive without a wildcard.

On an individual **variant** — `enum Event { Connected, #[non_exhaustive]
Disconnected { reason: String } }` — only that one variant loses
downstream literal construction and exhaustive destructuring; the enum's
other variants, and the enum as a whole, are otherwise ordinary. This is
the tool for letting a single variant grow new fields later without
marking every variant, or the whole enum, `#[non_exhaustive]`.

`#[non_exhaustive]` never affects reading fields, calling methods, or
matching a specific variant already known to exist — it only ever removes
the *assumption of completeness* a plain struct literal or an
un-wildcarded `match` would otherwise make. The compiler mechanism this
attribute interacts with — why exhaustiveness matters in the first
place — is covered on the
[Exhaustiveness checking](../../concepts/pattern-matching/exhaustiveness-checking.md)
concept page.

## Usage examples

### Restricting external struct-literal construction

```
#[non_exhaustive] // <- downstream crates can't build this with a plain struct literal
pub struct Config {
    pub retries: u32,
    pub timeout_ms: u32,
}

impl Config {
    pub fn new(retries: u32, timeout_ms: u32) -> Self {
        Config { retries, timeout_ms } // <- fine here: plain literals still work inside the defining crate
    }
}
```

### Designing a public API

A payment library's variant list is a natural candidate for future
growth as new providers get added, so the enum is marked
`#[non_exhaustive]` up front — before any downstream crate has a chance
to write a `match` that a later variant would silently break.

```
#[non_exhaustive] // <- tells downstream crates more variants may arrive without a breaking release
pub enum PaymentMethod {
    Card,
    BankTransfer,
}

pub struct RefundRequest {
    pub amount_cents: u64,
}

impl RefundRequest {
    pub fn new(amount_cents: u64) -> Self {
        RefundRequest { amount_cents }
    }
}
```

Adding a variant to a published, non-`#[non_exhaustive]`
enum is a breaking change for every downstream exhaustive `match`; the
[API Guidelines' future-proofing section](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends applying `#[non_exhaustive]` to both enums and structs a
library expects to grow, decided at first publish rather than retrofitted
after a breaking release already shipped.

### Branching on data (pattern matching)

Downstream code consuming a `#[non_exhaustive]` enum must add a wildcard
arm even when it currently lists every known variant, because the
compiler won't treat the match as complete without one.

```
// Defined in an upstream crate:
// #[non_exhaustive]
// pub enum PaymentMethod { Card, BankTransfer }

fn describe(method: &PaymentMethod) -> &'static str {
    match method {
        PaymentMethod::Card => "card",
        PaymentMethod::BankTransfer => "bank transfer",
        _ => "unsupported payment method", // <- required here: #[non_exhaustive] blocks treating this as exhaustive
    }
}
```

Without the wildcard arm, this fails to compile with an
error naming `PaymentMethod` as non-exhaustive, even though every variant
currently defined is already handled — the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
specifies this as deliberate: the wildcard is what lets the upstream
crate add a variant later without breaking this function.

## Explanation (Embedded)

`#[non_exhaustive]` is a pure compile-time attribute affecting only
downstream construction/matching rules, so it applies identically to a
`#![no_std]` HAL crate's public types. It's a natural fit for exactly the
kind of enum a hardware-abstraction crate tends to publish early and
grow later: a fault/error code enum describing what went wrong on a
peripheral (bus timeout, parity error, overrun, ...) is rarely complete
on day one — a HAL author who discovers a new distinguishable fault
condition in a later chip revision, or simply didn't enumerate every case
the silicon can report at first release, wants to add a variant without
that being a breaking change for every downstream `match`. Marking the
enum `#[non_exhaustive]` from its first published version keeps that door
open.

## Usage examples (Embedded)

### A HAL's fault-code enum left open for future variants

```
#[non_exhaustive] // <- lets this HAL add new fault variants later without a semver break
pub enum I2cFault {
    BusTimeout,
    ArbitrationLost,
    NackOnAddress,
}
```

### Downstream firmware handling the fault enum with a required wildcard

```
// Defined in the HAL crate:
// #[non_exhaustive]
// pub enum I2cFault { BusTimeout, ArbitrationLost, NackOnAddress }

fn recover_from(fault: &I2cFault) {
    match fault {
        I2cFault::BusTimeout => { /* reset the peripheral */ }
        I2cFault::ArbitrationLost => { /* retry the transaction */ }
        I2cFault::NackOnAddress => { /* mark the device unresponsive */ }
        _ => { /* unknown fault added in a later HAL version: fall back to a full peripheral reset */ }
    }
}
```

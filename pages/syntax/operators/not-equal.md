---
title: "!="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["=="]
see_also: ["=="]
---

## Explanation

`!=` tests inequality — the negation of `==`. It's provided automatically
by `std::cmp::PartialEq` (a single trait supplies both `eq` and, by
default, `ne` as `!self.eq(other)`); a type essentially never needs to
implement `!=` separately from `==`.

```
if a != b { ... }
```

## Basic usage example

```
let state = 3;
let ready = state != 0; // <- `!=` tests for inequality
```

## Best practices & deeper information

### Scenario: Validating input

A guard clause that rejects a sentinel or placeholder value reads more
directly with `!=` than with a negated `==`, and is a common first line
of defense before an input is used.

```
fn set_channel(channel: u8) -> Result<(), &'static str> {
    if channel != 0 { // <- `!=` guards against the reserved "unset" sentinel
        Ok(())
    } else {
        Err("channel 0 is reserved and cannot be assigned")
    }
}

assert!(set_channel(0).is_err());
assert!(set_channel(4).is_ok());
```

**Why this way:** `channel != 0` reads as "channel is set" more directly
than `!(channel == 0)`, and the compiler treats them identically since
`!=` is `PartialEq::ne`, not a separate check — see
[`==`](equal-equal.md) for the fuller treatment of the underlying trait
and its derive/impl tradeoffs.

## Embedded Rust Notes

**Full support.** Same trait as [`==`](equal-equal.md), no `std`
dependency.

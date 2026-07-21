---
title: "priv"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["pub"]
see_also: ["pub"]
---

## Explanation

`priv` has been reserved since the 2015 edition, but unlike most of the
purely speculative entries in this section, it has an actual small
history: `priv` was briefly **real, usable syntax** in pre-1.0 Rust. In
that early design, visibility worked the other way around from today —
items were implicitly *public* by default, and an explicit `priv`
keyword was how you marked something private, the exact inverse role
[`pub`](pub.md) plays now. That design was reversed before Rust 1.0:
today, items are private by default and `pub` is the marker you add to
opt into visibility. `priv` was removed as functioning syntax but kept
reserved, in case some future visibility refinement wants to reuse the
word — no concrete proposal currently claims it.

Using `priv` as an ordinary identifier is a compile error today. The
raw-identifier form `r#priv` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### Using the raw-identifier escape hatch

```
let priv = 5;     // error: expected identifier, found reserved keyword `priv`
let r#priv = 5;   // ok: the raw-identifier form escapes the reservation
```

### Designing a public API

Today's visibility model expresses exactly what `priv` used to express,
just with the default flipped: a struct's fields stay private unless
explicitly marked `pub`, which is what lets a type keep its invariants
enforced through a constructor instead of allowing direct field
mutation.

```
pub struct Account {
    balance: i64, // private by default — no `priv` needed, this is the current default
}

impl Account {
    pub fn new(opening_balance: i64) -> Self {
        Account { balance: opening_balance.max(0) } // <- invariant enforced here, not by the field
    }

    pub fn balance(&self) -> i64 {
        self.balance
    }
}
```

If `balance` were public, nothing would stop external
code from setting it to a negative value directly; keeping it private
(today's default, once `priv`'s job before 1.0) and exposing only a
validating constructor and a read accessor is the same "invalid states
unrepresentable" discipline the API Guidelines and Effective Rust both
build their privacy advice around.

## Embedded Rust Notes

**Full support.** Keyword reservation is a lexer-level concept, identical
in `#![no_std]` and hosted Rust alike.

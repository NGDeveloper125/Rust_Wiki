---
title: "-="
kind: operator
embedded_support: full
groups: [Arithmetic, Basics]
related_concepts: [Operator overloading]
related_syntax: ["-"]
see_also: ["-"]
---

## Explanation

`-=` subtracts the right operand from the left in place, overloadable via
`std::ops::SubAssign`.

See [`+=`](plus-equals.md) for the general notes on compound assignment
operators (mutable place required, potentially distinct impl from the
non-assigning operator).

## Usage examples

### Subtracting a value from a variable in place

```
let mut x = 10;
x -= 3; // <- subtracts 3 from `x` in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `-=` assigns in place.

### Modifying an existing object

Decrementing an account balance through a `&mut` reference is a typical
use of `-=` — validate first, then update the field in place.

```
struct Account {
    balance: i64, // cents
}

fn withdraw(account: &mut Account, cents: i64) -> Result<(), &'static str> {
    if cents > account.balance {
        return Err("insufficient funds");
    }
    account.balance -= cents; // <- subtracts `cents` from the balance in place
    Ok(())
}
```

Checking the invariant (`cents > balance`) before the
`-=` keeps the balance from ever going negative, and updating in place
through `&mut` avoids a separate read-modify-write statement — see
[`+=`](plus-equals.md) for the compound-assignment notes shared across
the whole family.

## Explanation (Embedded)

`SubAssign` lives in `core::ops`, so `-=` behaves identically under
`#![no_std]`. The relevant nuance is the same one covered on
[`-`](minus.md): a release build ships with overflow checks off, so a
`-=` that underflows wraps silently rather than panicking, and a device
already deployed can't be recompiled in debug to catch it. Where the
right-hand side is a runtime value that isn't guaranteed to stay smaller
than the left — decrementing a countdown, a remaining-capacity counter, or
a fuel/charge gauge — it's worth guarding the subtraction (or using
`checked_sub`/`saturating_sub` explicitly) rather than trusting bare `-=`
to fail loudly the way a debug build would.

## Usage examples (Embedded)

### Decrementing a countdown timer without an unchecked underflow

```
struct Watchdog {
    remaining: u16,
}

impl Watchdog {
    fn tick(&mut self) -> bool {
        if self.remaining == 0 {
            return false; // guard first: `-=` on 0 here would panic (debug) or wrap (release)
        }
        self.remaining -= 1; // <- `-=` decrements in place, now safe from underflow
        true
    }
}
```

### Draining a charge counter with a saturating alternative

```
struct Battery {
    milliamp_hours: u32,
}

impl Battery {
    fn drain(&mut self, used: u32) {
        self.milliamp_hours = self.milliamp_hours.saturating_sub(used); // stands in for `-=`, floors at 0 instead of wrapping
    }
}
```

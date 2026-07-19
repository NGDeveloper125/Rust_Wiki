---
title: "-="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["-"]
see_also: ["-"]
---

## Explanation

`-=` subtracts the right operand from the left in place, overloadable via
`std::ops::SubAssign`.

```
let mut x = 5;
x -= 2; // x is now 3
```

See [`+=`](plus-equals.md) for the general notes on compound assignment
operators (mutable place required, potentially distinct impl from the
non-assigning operator).

## Basic usage example

```
let mut x = 10;
x -= 3; // <- subtracts 3 from `x` in place
```

**Restriction:** the left-hand side must be a mutable binding
(`let mut`) — `-=` assigns in place.

## Best practices & deeper information

### Scenario: Modifying an existing object

Decrementing an account balance through a `&mut` reference is a typical
use of `-=` — validate first, then update the field in place.

```
struct Account {
    balance: f64,
}

fn withdraw(account: &mut Account, amount: f64) -> Result<(), &'static str> {
    if amount > account.balance {
        return Err("insufficient funds");
    }
    account.balance -= amount; // <- subtracts `amount` from the balance in place
    Ok(())
}
```

**Why this way:** checking the invariant (`amount > balance`) before the
`-=` keeps the balance from ever going negative, and updating in place
through `&mut` avoids a separate read-modify-write statement — see
[`+=`](plus-equals.md) for the compound-assignment notes shared across
the whole family.

## Embedded Rust Notes

**Full support.** `SubAssign` lives in `core::ops` — no `std` dependency.

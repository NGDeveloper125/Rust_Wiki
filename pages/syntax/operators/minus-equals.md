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

## Embedded Rust Notes

**Full support.** `SubAssign` lives in `core::ops` — no `std` dependency.

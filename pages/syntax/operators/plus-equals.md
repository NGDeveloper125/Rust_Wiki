---
title: "+="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: ["+"]
see_also: ["+"]
---

## Explanation

`+=` adds the right operand to the left in place, overloadable via
`std::ops::AddAssign`:

```
let mut x = 5;
x += 1; // x is now 6
```

`x += 1` is not always exactly sugar for `x = x + 1` — types can implement
`AddAssign` differently from `Add` (e.g. to mutate in place without an
extra allocation, which matters for types like `String` or `Vec`), though
for simple numeric types the two behave identically. The left operand
must be a mutable place — `x` must be declared `let mut x`.

## Basic usage example

```
let mut x = 5;
x += 1; // <- `+=` adds the right operand into `x` in place
```

**Restriction:** the left operand must be a mutable place — `x` has to be
declared with `let mut x`, or this won't compile.

## Best practices & deeper information

### Scenario: Modifying an existing object

An account balance is a natural home for `+=` — the balance is a field
mutated in place each time a deposit posts, rather than a whole new
`Account` being constructed per transaction.

```
struct Account {
    balance: i64, // cents
}

impl Account {
    fn deposit(&mut self, cents: i64) {
        self.balance += cents; // <- `+=` mutates the field in place
    }
}

let mut checking = Account { balance: 10_000 };
checking.deposit(2_500);
checking.deposit(750);
assert_eq!(checking.balance, 13_250);
```

**Why this way:** a `&mut self` method that uses `+=` on its own field
keeps the mutation local and auditable — the alternative of returning a
new `Account` on every deposit would work but adds allocation and
ceremony for a value type that's meant to change over its lifetime, per
the mutability guidance in [the Book](https://doc.rust-lang.org/book/ch05-03-method-syntax.html).

### Scenario: Working with collections

Accumulating a running total while iterating is the textbook use of
`+=`; it's worth contrasting with re-binding a fresh `let total = ...` on
every loop pass, which doesn't compile without `mut` and wouldn't express
"the same total, growing" as directly even if it did.

```
let orders = [1200, 450, 899, 3000]; // cents

let mut total = 0; // <- `mut` binding `+=` will mutate
for cents in orders {
    total += cents; // <- `+=` folds each order into the running total
}
assert_eq!(total, 5549);

// Equivalent, more idiomatic for a one-shot fold:
let total: i32 = orders.iter().sum();
```

**Why this way:** the explicit `for` loop with `+=` is the clearest form
when the accumulation is interleaved with other per-item work; when
summing is the *only* thing happening, `Iterator::sum` from the
[standard library docs](https://doc.rust-lang.org/std/iter/trait.Sum.html)
expresses the same intent with no mutable state at all.

## Embedded Rust Notes

**Full support.** `AddAssign` lives in `core::ops` — no `std` dependency.

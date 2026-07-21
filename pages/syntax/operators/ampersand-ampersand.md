---
title: "&&"
kind: operator
embedded_support: full
groups: [Logical, Basics]
related_concepts: []
related_syntax: ["||", "!"]
see_also: ["||"]
---

## Explanation

`&&` is short-circuiting logical AND between two `bool` values, as in
`a > 0 && b > 0`.

"Short-circuiting" means the right operand is only evaluated if the left
is `true` — important when the right side has side effects or could
panic (`x.is_some() && x.unwrap() > 0`). Unlike `&`, `&&` is **not**
overloadable — it only ever works on `bool` and always short-circuits;
there's no trait to implement to change its behavior for a custom type.

## Usage examples

### Short-circuiting two conditions

```
let a = 3;
let b = 5;
let both_positive = a > 0 && b > 0; // <- `&&` short-circuits: `b > 0` only runs if `a > 0` is true
```

**Restriction:** `&&` only works on `bool` operands and can't be
overloaded for custom types — unlike `&`, there is no trait to implement
to change its behavior.

### Validating input

Chaining several guard conditions with `&&` reads as one sentence and
short-circuits before any check that would panic on data the earlier
checks already rejected.

```
struct SignupForm {
    username: String,
    age: u32,
}

fn is_valid(form: &SignupForm) -> bool {
    !form.username.is_empty()
        && form.username.chars().next().unwrap().is_alphabetic() // <- only runs if username isn't empty
        && form.age >= 18                                        // <- `&&` short-circuits before this
}
```

Ordering the cheap, panic-free checks first and letting
`&&` short-circuit means the `unwrap()` on the first character never runs
against an empty string — see [`||`](pipe-pipe.md) for the OR counterpart,
used when any single condition failing should reject the input.

## Embedded Rust Notes

**Full support.** Built into the language, not a trait — no `std`
dependency.

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

## Explanation (Embedded)

`&&` behaves identically in `#![no_std]` — it's core-language
short-circuit boolean logic, so there's no runtime or allocator
involved and nothing about it changes on a bare-metal target. What is
worth calling out is how often it shows up in embedded control flow
specifically: guard conditions that combine "the hardware isn't ready
yet" with "give up after some bound" are a constant pattern in polling
and retry loops, and `&&`'s short-circuiting means the retry counter is
only checked once the hardware condition has already failed — the
counter never gets touched, and no extra polling happens, once the
hardware reports ready.

## Usage examples (Embedded)

### Guarding a retry loop against a non-responsive sensor

```
const MAX_RETRIES: u8 = 5;

fn sensor_ready() -> bool {
    false // placeholder: real code polls a status register bit
}

fn read_sensor_raw() -> u16 {
    0
}

fn delay_ms(_ms: u32) {}

fn read_sensor_with_retry() -> Option<u16> {
    let mut retries = 0;
    while !sensor_ready() && retries < MAX_RETRIES {
        // <- `&&`: `retries` is only checked once `sensor_ready()` has already failed
        retries += 1;
        delay_ms(10);
    }
    if sensor_ready() {
        Some(read_sensor_raw())
    } else {
        None
    }
}
```

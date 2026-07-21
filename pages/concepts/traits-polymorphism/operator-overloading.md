---
title: "Operator overloading (std::ops traits)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Polymorphism"]
related_syntax: ["+", "*", "==", "[ ]"]
see_also: ["Traits"]
---

## Explanation

Operators like `+`, `[]`, and `*` aren't special-cased into the
language for user-defined types — they're ordinary trait methods. Most
live in `std::ops` (`Add`, `Index`, `Mul`, and so on); the comparison
operators (`==`, `<`, …) are the exception, coming from `std::cmp`
(`PartialEq`, `PartialOrd`). Any type can implement the relevant trait to
give meaning to that operator for itself — for instance, implementing
`std::ops::Add` for a `Point` struct gives meaning to `p1 + p2`.

Once implemented, `p1 + p2` calls this `add` method directly — the
operator syntax is just sugar over the trait method call, resolved at
compile time. Because it's an ordinary trait, the same rules apply as to
any other trait: you can only implement it for types you own (or foreign
traits on foreign types via [the newtype pattern](../types-data-modeling/the-newtype-pattern.md)),
and the compiler enforces that the types involved actually make sense
together (e.g. `Add`'s associated `Output` type must be specified
explicitly, so `Point + Point` returning something other than `Point` is
possible but has to be deliberate).

## Basic usage example

```
use std::ops::Add;

struct Point { x: i32, y: i32 }

impl Add for Point { // <- gives `+` its meaning for Point
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

let p = Point { x: 1, y: 2 } + Point { x: 3, y: 4 }; // <- calls Point::add
```

## Best practices & deeper information

### Scenario: Creating a new object

Combining two `Money` values into a new one reads naturally as `a + b`
once `Add` is implemented — the operator becomes the constructor for the
summed value.

```
use std::ops::Add;

#[derive(Clone, Copy)]
struct Money { cents: i64 }

impl Add for Money { // <- gives `+` its meaning: combining two Money values into a new one
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money { cents: self.cents + other.cents }
    }
}

let total = Money { cents: 500 } + Money { cents: 250 }; // <- builds a new Money via `+`
```

**Why this way:**
[`std::ops::Add`](https://doc.rust-lang.org/std/ops/trait.Add.html) exists
precisely so a value can be combined with `+` instead of a
differently-named method like `.plus()` — once implemented, `Money`
composes with any code already written in terms of `+`.

### Scenario: Designing a public API

Operator overloading is idiomatic only when the operator's usual meaning
genuinely applies — implementing `Add` for something that isn't really
"addable" surprises every caller who reads `a + b` and expects
arithmetic-like behavior.

```
struct Config { retries: u32 }

// AVOID: overloads `+` but the meaning is surprising (not addition)
impl std::ops::Add for Config {
    type Output = Config;
    fn add(self, other: Config) -> Config {
        Config { retries: self.retries.max(other.retries) }
    }
}

// PREFER: a named method when the operation isn't actually arithmetic
impl Config {
    fn merged_with(self, other: Config) -> Config { // <- no operator trait involved
        Config { retries: self.retries.max(other.retries) }
    }
}
```

**Why this way:** the
[API Guidelines' C-OVERLOAD](https://rust-lang.github.io/api-guidelines/predictability.html)
is explicit that operators come with strong expectations — an operator
should only be overloaded for an operation that genuinely resembles what
that operator means arithmetically, and a named method used otherwise.

## Explanation (Embedded)

`core::ops` and `core::cmp` provide the exact same traits as their `std`
re-exports, so operator overloading works fully under `#![no_std]` with
no substitute and no caveat. The scenario worth grounding this in is a
register-flags newtype: a driver that models a peripheral's control
register as a plain `u8`/`u16`/`u32` gets no protection against, say,
adding two register values together as if they were numbers instead of
combining their bits. Wrapping the raw integer in a newtype and
implementing `BitOr`/`BitAnd`/`BitOrAssign` (and friends) on the newtype
gives it the operators that actually make sense for a flag set (`|` to
combine flags, `&` to test them) while withholding the ones that don't
(no `Add` impl at all, since summing two register values isn't a
meaningful operation). [`|`'s embedded
notes](../../syntax/operators/pipe.md) and [`&`'s embedded
notes](../../syntax/operators/ampersand.md) already cover assembling and
testing register bits held in a plain integer; the angle here is
different and complementary — not how `|`/`&` are *used* on register
bits, but how a driver author *implements* the trait so the newtype gets
that same bitwise vocabulary as first-class, named operators instead of
raw integer arithmetic.

## Basic usage example (Embedded)

```
use core::ops::BitOr;

#[derive(Clone, Copy)]
struct ControlFlags(u8);

impl BitOr for ControlFlags { // <- gives `|` its meaning: combining two register flag sets
    type Output = ControlFlags;
    fn bitor(self, other: ControlFlags) -> ControlFlags {
        ControlFlags(self.0 | other.0)
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A control-register newtype implementing `BitOr`/`BitAnd` lets a driver
combine and test flags with `|`/`&` directly on the domain type, instead
of unwrapping to a raw integer at every call site.

```
use core::ops::{BitAnd, BitOr};

#[derive(Clone, Copy, PartialEq)]
struct ControlFlags(u8);

const ENABLE: ControlFlags = ControlFlags(1 << 0);
const RX_INTERRUPT: ControlFlags = ControlFlags(1 << 3);

impl BitOr for ControlFlags { // <- combining two flag sets into one
    type Output = ControlFlags;
    fn bitor(self, other: ControlFlags) -> ControlFlags {
        ControlFlags(self.0 | other.0)
    }
}

impl BitAnd for ControlFlags { // <- testing whether a flag is present
    type Output = ControlFlags;
    fn bitand(self, other: ControlFlags) -> ControlFlags {
        ControlFlags(self.0 & other.0)
    }
}

let cr1 = ENABLE | RX_INTERRUPT; // <- reads as flag combination, not raw integer OR
let rx_enabled = (cr1 & RX_INTERRUPT) == RX_INTERRUPT;
```

**Why this way:** implementing `BitOr`/`BitAnd` directly on the newtype
means every call site combines and tests flags through the same named
operators a raw `u8` would use, while the newtype still blocks
nonsensical operations (no `Add` impl exists, so `cr1 + RX_INTERRUPT`
is a compile error) — [`std::ops::BitOr`](https://doc.rust-lang.org/std/ops/trait.BitOr.html)
exists precisely so bit-combination reads as `|` rather than a
differently-named method, and a newtype is what lets a driver keep that
readability while still being a distinct type from a bare integer.

### Scenario: Designing a public API

Operator overloading on a register-flags type should stop at the
operators whose bitwise meaning genuinely applies — implementing
`BitOr`/`BitAnd`/`BitOrAssign` is idiomatic, but reaching for `Add` or
`Mul` on the same newtype just because the underlying storage is
numeric invites callers to write nonsensical register arithmetic.

```
#[derive(Clone, Copy)]
struct ControlFlags(u8);

// AVOID: Add doesn't mean anything for a flag set — two control registers
// were never meant to be summed like integers.
// impl std::ops::Add for ControlFlags { ... }

// PREFER: only the operators whose bitwise meaning genuinely applies
impl core::ops::BitOrAssign for ControlFlags {
    fn bitor_assign(&mut self, other: ControlFlags) { // <- `|=` sets additional flags in place
        self.0 |= other.0;
    }
}
```

**Why this way:** the [API Guidelines'
C-OVERLOAD](https://rust-lang.github.io/api-guidelines/predictability.html)
principle — only overload an operator when its usual meaning genuinely
applies — is just as real for a register-flags newtype as for any other
type; implementing `Add` on a flags type would compile without error but
mislead every caller who reads `flags_a + flags_b` and expects arithmetic
rather than a nonsensical bit-pattern sum.

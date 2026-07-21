---
title: "Tuple structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling"]
related_syntax: [struct]
see_also: ["Structs", "The newtype pattern"]
---

## Explanation

A tuple struct is a struct whose fields are identified by position rather
than by name — `struct Point(f64, f64);` and a value of it, `Point(1.0, 2.0)`,
are accessed positionally rather than by field name.

It's a middle ground between a plain tuple `(f64, f64)` (no type identity
of its own — any two `f64`s are interchangeable with it) and a full named
struct (every field has an explicit name). A tuple struct gives a set of
values a distinct type — `Point` is not the same type as `(f64, f64)` and
the two can't be used interchangeably — while keeping field access
positional (`p.0`, `p.1`) for cases where names would add little.

The single-field case, `struct Meters(f64);`, is common enough to have
its own name — see [the newtype pattern](the-newtype-pattern.md) — where
the point isn't brevity but using the type system to prevent mixing up
values that happen to share an underlying representation.

## Basic usage example

```
struct Point(f64, f64); // <- fields identified by position, not by name

let p = Point(1.0, 2.0);
println!("{}", p.0); // <- positional access: p.0, p.1
```

## Best practices & deeper information

### Scenario: Designing a public API

A tuple struct is the right call when field order is self-evident and
spelling out names would just repeat what the position already says —
here, a color's three positional bytes.

```
struct Rgb(u8, u8, u8); // <- three positional fields; order IS the meaning (red, green, blue)

fn paint(color: Rgb) {
    println!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);
}

paint(Rgb(255, 87, 34));
```

**Why this way:** favor clarity over ceremony — a tuple struct is right
while field meaning is obvious from position; once it stops being
obvious, switch to a named struct; and once the goal shifts from
readability to *preventing* values of the same underlying type from being
mixed up, reach for [the newtype pattern](the-newtype-pattern.md)
instead.

### Scenario: Creating a new object

Even a tuple struct benefits from a named constructor once the positions
mean something specific enough that spelling them out at every call site
would be easy to get wrong.

```
struct Version(u32, u32, u32); // <- positional: major, minor, patch

impl Version {
    fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version(major, minor, patch) // <- constructor documents what each position means
    }
}

let v = Version::new(1, 4, 0);
println!("{}.{}.{}", v.0, v.1, v.2);
```

**Why this way:** the API Guidelines'
[C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
convention applies just as much to tuple structs as to named ones — a
`new()` with named parameters is far less error-prone at the call site
than three bare positional arguments to the tuple constructor directly.

## Explanation (Embedded)

The mechanism is identical to hosted Rust — positional fields instead of
named ones — and the same "brevity vs. the newtype pattern" trade-off
from the classic explanation applies unchanged in `#![no_std]`. The
genuinely embedded-flavored case is a small value where the position
already carries all the meaning: a pair of raw register values read
together (a low/high half of a 32-bit counter split across two 16-bit
registers, say) or a sensor's raw axis readings, where `reading.0`,
`reading.1`, `reading.2` reading as "first axis, second axis, third axis"
costs nothing over naming them `x`, `y`, `z`, and a named struct would be
scarcely more informative.

## Basic usage example (Embedded)

```
struct Axes(i16, i16, i16); // <- three positional fields: a 3-axis accelerometer's raw readings

let sample = Axes(120, -34, 998); // roughly 1g on the third axis, at rest
println!("{}", sample.2); // <- positional access: sample.0, .1, .2
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A register pair split across two hardware registers (a timer's low and
high halves) is a natural tuple struct — the order *is* the meaning, and
naming the fields `low`/`high` would repeat what the position already
says.

```
struct CounterHalves(u16, u16); // <- (low, high): position already documents which half is which

fn combine(halves: CounterHalves) -> u32 {
    (halves.1 as u32) << 16 | halves.0 as u32 // <- .0 is low, .1 is high, exactly as declared
}

let count = combine(CounterHalves(0xBEEF, 0x0001));
```

**Why this way:** once two positional values start meaning genuinely
different things that a reader could get backwards (as opposed to
"obviously the low half comes first"), that's the signal to switch to a
named struct or to [the newtype pattern](the-newtype-pattern.md) instead
— but a two-register split where the datasheet itself defines the order
is exactly the case where positional access costs nothing in clarity.

### Scenario: Creating a new object

Even a small tuple struct benefits from a constructor once its positions
correspond to something a caller could otherwise get wrong, like which
raw sensor axis maps to which physical direction on the board.

```
struct Axes(i16, i16, i16); // <- positional: x, y, z raw accelerometer counts

impl Axes {
    fn from_raw(x: i16, y: i16, z: i16) -> Self {
        Axes(x, y, z) // <- constructor documents which position is which axis
    }
}

let sample = Axes::from_raw(120, -34, 998);
```

**Why this way:** the same
[C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
convention applies here — named constructor parameters are far less
error-prone at the call site than three bare positional arguments to the
tuple constructor directly, especially where an axis mix-up would only
show up as a physically-wrong sign at runtime.

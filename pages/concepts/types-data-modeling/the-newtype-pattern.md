---
title: "The newtype pattern"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models", "Unique to Rust"]
related_syntax: [struct]
see_also: ["Tuple structs", "Type aliases", "The orphan rule & coherence"]
---

## Explanation

The newtype pattern wraps an existing type in a single-field tuple struct
to give it a distinct identity — `struct Meters(f64);` and
`struct Seconds(f64);` are a typical pair.

Even though both wrap `f64`, `Meters` and `Seconds` are different types
to the compiler — passing a `Seconds` value where `Meters` is expected is
a compile error, catching a whole category of unit-confusion bugs
(the kind that has, famously, destroyed real spacecraft) that a plain
type alias (which is fully interchangeable with what it aliases) would
not catch at all.

Newtypes serve a second, unrelated purpose too: Rust's
[orphan rule](../traits-polymorphism/the-orphan-rule-and-coherence.md)
forbids implementing a foreign trait on a foreign type directly (e.g. you
can't `impl Display for Vec<T>` from outside the crate that defines
either), but wrapping the foreign type in your own newtype gives you a
type you *do* own, on which you're free to implement whatever traits you
like. This combination — meaningful type safety plus a way around the
orphan rule — is common enough that it's considered one of Rust's
signature idioms rather than a niche trick.

## Basic usage example

```
struct Meters(f64); // <- a distinct type, not just another name for f64

fn print_distance(d: Meters) {
    println!("{} m", d.0); // <- the wrapped value is accessed via .0
}

print_distance(Meters(5.0));
// print_distance(5.0); // <- would not compile: f64 is not a Meters
```

**Restriction:** the wrapper gets no behavior for free — arithmetic
operators, trait impls, and methods that `f64` has don't automatically
apply to `Meters`; each one needed on the newtype has to be implemented
explicitly (e.g. `impl Add for Meters`).

## Best practices & deeper information

### Scenario: Designing a public API

Two parameters of the same primitive type, back to back, are exactly the
shape that invites an accidental argument swap that compiles cleanly and
misbehaves at runtime. Wrapping each in its own newtype turns that swap
into a compile error.

```
struct UserId(u64);  // <- distinct from OrderId even though both wrap u64
struct OrderId(u64);

fn refund(user: UserId, order: OrderId) { // <- swapping the argument order is now a compile error
    println!("refunding order {} for user {}", order.0, user.0);
}

refund(UserId(42), OrderId(1001));
// refund(OrderId(1001), UserId(42)); // <- would not compile: types don't match
```

**Why this way:** [Effective Rust](https://effective-rust.com/) covers
newtypes as a primary defense against this exact class of bug — two
`u64` parameters are indistinguishable to the compiler, but `UserId` and
`OrderId` are not, so a mixed-up call site is caught before it ships.

### Scenario: Converting between types

Implementing `From`/`Into` in both directions gives a newtype
conventional wrapping and unwrapping, instead of an ad-hoc method name
every newtype in the codebase would otherwise invent independently.

```
struct UserId(u64);

impl From<u64> for UserId {
    fn from(raw: u64) -> Self { // <- wrapping goes through From, not a custom method name
        UserId(raw)
    }
}

impl From<UserId> for u64 {
    fn from(id: UserId) -> Self { // <- unwrapping is just as conventional as wrapping
        id.0
    }
}

let id: UserId = 42.into();  // <- wrap
let raw: u64 = id.into();    // <- unwrap
```

**Why this way:** the API Guidelines'
[C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv)
recommend the standard `From`/`Into` traits over one-off conversion
methods — doing so lets the newtype interoperate with any API already
written against `Into`, for free.

## Explanation (Embedded)

The newtype pattern is one of the most load-bearing idioms in embedded
Rust specifically, because embedded code is saturated with values that
are "just" an integer at the machine level — a raw ADC count, a GPIO pin
number, a register's raw bit pattern — but mean very different things
depending on where they came from. `struct Millivolts(u16)` and
`struct AdcCount(u16)` are both `u16` underneath, but wrapping each in its
own newtype means a calibrated voltage can never be passed where a raw,
uncalibrated ADC reading was expected (or vice versa) without a compile
error — a mistake that, on hardware, tends to surface as a silently wrong
reading rather than a crash, making it exactly the kind of bug worth
catching at compile time instead of in the field.

Crucially, this safety costs nothing at runtime. As covered on
[Zero-cost abstractions](../philosophy-principles/zero-cost-abstractions.md),
a single-field newtype has identical layout to its inner field once
`#[repr(transparent)]` is applied (see
[`#[repr(...)]`](../../syntax/attributes/repr.md) for the layout guarantee
this makes), so `Millivolts(u16)` is exactly two bytes, passes through an
`extern "C"` HAL boundary exactly like a raw `u16` would, and the
wrapping/unwrapping disappears entirely from the compiled code. On a
target with a few kilobytes of RAM total, being able to add a whole
category of type safety without spending a single byte or cycle on it is
precisely why the pattern shows up throughout `embedded-hal`-ecosystem
crates: raw register values, pin identifiers, and peripheral handles are
routinely wrapped in newtypes for exactly this reason.

## Basic usage example (Embedded)

```
struct Millivolts(u16); // <- distinct from a raw ADC count, even though both are u16 underneath
struct AdcCount(u16);

fn to_millivolts(raw: AdcCount, vref_mv: u16) -> Millivolts {
    Millivolts((raw.0 as u32 * vref_mv as u32 / 4095) as u16)
}

let reading = to_millivolts(AdcCount(2048), 3300);
// print_voltage(AdcCount(2048)); // <- would not compile: an AdcCount is not a Millivolts
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

Two `u16` parameters back to back — a raw pin number and a debounce
delay, say — is exactly the shape that invites an accidental swap;
wrapping each in its own newtype turns the swap into a compile error
instead of a hardware misconfiguration.

```
struct PinNumber(u8);  // <- distinct from DebounceMs even though both wrap small integers
struct DebounceMs(u16);

fn configure_button(pin: PinNumber, debounce: DebounceMs) { // <- swapping the argument order is now a compile error
    let _ = (pin.0, debounce.0);
}

configure_button(PinNumber(4), DebounceMs(20));
// configure_button(DebounceMs(20), PinNumber(4)); // <- would not compile: types don't match
```

**Why this way:** [Effective Rust](https://effective-rust.com/) covers
this exact defense — two integer parameters of the same primitive type
are indistinguishable to the compiler, but distinct newtypes are not, so
a mixed-up call site at the hardware boundary is caught before it ever
reaches the board.

### Scenario: Converting between types

A raw ADC reading needs to become a calibrated voltage at exactly one
place in the code; implementing `From` for that conversion keeps wrapping
and unwrapping conventional instead of inventing a one-off method name
per newtype.

```
struct AdcCount(u16);
struct Millivolts(u32);

impl From<AdcCount> for Millivolts {
    fn from(raw: AdcCount) -> Self { // <- calibration math lives in one conventional place
        Millivolts(raw.0 as u32 * 3300 / 4095)
    }
}

let voltage: Millivolts = AdcCount(2048).into(); // <- .into() reads as "convert", not a custom method name
```

**Why this way:** using the standard `From`/`Into` traits rather than a
custom `to_millivolts()`-style method lets the conversion interoperate
with any generic code already written against `Into`, per the API
Guidelines'
[C-CONV-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#conversions-use-the-standard-traits-from-asref-asmut-c-conv).

### Scenario: Bit manipulation and flags

Wrapping a peripheral's raw status register in a newtype lets
bit-testing methods live on a type that documents what the bits mean,
instead of scattering raw shifts and masks across the code that reads the
register.

```
#[repr(transparent)] // <- zero-cost: identical layout to the raw u32 register value
struct StatusRegister(u32);

impl StatusRegister {
    fn is_busy(&self) -> bool {
        self.0 & 0b1 != 0 // <- bit 0: busy flag
    }
    fn has_error(&self) -> bool {
        self.0 & 0b10 != 0 // <- bit 1: error flag
    }
}

let status = StatusRegister(0b10); // read from a memory-mapped register in real code
assert!(status.has_error());
```

**Why this way:** naming the bit-testing operations as methods on the
newtype documents the register's bit layout at the call site
(`status.has_error()` instead of a bare `raw & 0b10 != 0` the reader has
to decode), while `#[repr(transparent)]` guarantees the wrapper adds no
size or cost over the raw `u32` it replaces.

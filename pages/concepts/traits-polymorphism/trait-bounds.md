---
title: "Trait bounds"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code", "Decoupling", "Generic Programming"]
related_syntax: [":", "+", where]
see_also: ["Traits", "Generics", "Supertraits"]
---

## Explanation

A trait bound constrains a generic type parameter to only the types that
implement a given trait, giving generic code exactly the guarantees it
needs to actually do something useful with a value of an otherwise
unknown type — for example, `fn largest<T: PartialOrd>(items: &[T]) -> &T`
constrains `T` so that `>` comparisons are allowed inside the function
body.

Without the bound, the compiler would have no basis for allowing `>` to
be used on values of type `T` inside the function body — `T` could be
anything. The bound `T: PartialOrd` says "whatever `T` ends up being, it
must support ordering comparisons," which the compiler then checks holds
at every call site.

This is the mechanism that lets Rust decouple code from concrete types
in a fully type-checked way: a function can depend on "any type that can
be compared" or "any type that can be displayed," rather than a specific
concrete type, and the compiler verifies both that the generic code only
uses operations the bound actually grants, and that every real type
passed in at a call site genuinely satisfies the bound. Multiple bounds
combine with `+` (`T: Clone + Debug`), and a `where` clause is available
for bounds that get too long to read comfortably inline.

## Basic usage example

```
fn largest<T: PartialOrd>(items: &[T]) -> &T { // <- bound: only types supporting `>` allowed
    let mut m = &items[0];
    for item in items {
        if item > m { m = item; }
    }
    m
}

largest(&[3, 7, 2]);
```

## Best practices & deeper information

### Scenario: Writing generic code

Once a generic function needs more than one bound, or the bound list gets
long, moving it to a `where` clause keeps the signature itself scannable
without changing what's required.

```
fn unique_sorted<T>(readings: &[T]) -> Vec<T>
where
    T: Ord + Clone, // <- bounds relocated to `where`, still constraining T the same way
{
    let mut sorted: Vec<T> = readings.to_vec(); // needs Clone
    sorted.sort();                              // needs Ord
    sorted.dedup();
    sorted
}

unique_sorted(&[3, 1, 3, 2, 1]);
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html)
recommends `where` once a function has multiple generic parameters each
carrying their own bounds — it separates "what the function does" (the
signature) from "what its inputs must support" (the bounds).

### Scenario: Designing a public API

A generic function should only require what its own body actually uses —
bounding it against a broader trait than necessary forces every caller's
type to satisfy requirements the function never touches.

```
// AVOID: over-constrained — Clone is required but never used
fn describe_avoid<T: Clone + std::fmt::Debug>(item: &T) -> String {
    format!("{item:?}")
}

// PREFER: bound only what the body needs
fn describe<T: std::fmt::Debug>(item: &T) -> String { // <- Debug is the only bound this fn needs
    format!("{item:?}")
}
```

**Why this way:** minimal bounds keep the function usable by the widest
range of types and make the signature an honest description of its
requirements — the
[API Guidelines' C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html)
names this directly: functions should minimize assumptions about their
parameters.

## Explanation (Embedded)

Trait bounds are the single most-used tool for writing hardware-agnostic
embedded code — more so than in most hosted Rust, because the
alternative (dynamic dispatch or an enum of every supported chip) tends
to cost either flash space or genuine flexibility. A function like
`fn blink<P: OutputPin>(pin: &mut P)` is bounded against an
`embedded-hal` trait rather than any vendor's concrete pin type, so the
same function body is monomorphized separately for every concrete pin
type it's called with — zero runtime dispatch, and it compiles against
any microcontroller whose HAL implements `OutputPin`. See
[`trait`](../../syntax/keywords/trait.md) for how `embedded-hal`'s traits
are declared in the first place; this page is about the bound that lets
generic driver code depend on them.

The bound also carries the trait's associated items along with it. Most
`embedded-hal` traits declare an associated `Error` type rather than a
fixed error enum (peripherals fail in vendor-specific ways), so a bounded
function's return type is typically `Result<(), P::Error>` — the bound
`P: OutputPin` is what makes `P::Error` a meaningful type at all inside
the function body. Combining bounds with `+` is common once a driver
needs more than one capability from the same type parameter, or two
different trait parameters entirely (a display driver bounded by both
`SpiBus` for the data line and `OutputPin` for a chip-select line), and a
`where` clause keeps that readable once the list grows.

## Basic usage example (Embedded)

```
trait OutputPin {
    type Error;
    fn set_high(&mut self) -> Result<(), Self::Error>;
    fn set_low(&mut self) -> Result<(), Self::Error>;
}

fn blink<P: OutputPin>(pin: &mut P, on: bool) -> Result<(), P::Error> { // <- bound: only pin types implementing OutputPin allowed
    if on { pin.set_high() } else { pin.set_low() }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A display driver needs two capabilities from its caller's hardware at
once — a data bus and a separate chip-select line — so it's bounded over
two `embedded-hal` traits, one per generic parameter, combined in a
`where` clause once the list is long enough to hurt readability inline.

```
fn write_frame<BUS, CS>(bus: &mut BUS, cs: &mut CS, frame: &[u8]) -> Result<(), BUS::Error>
where
    BUS: SpiBus,        // <- bound: the data line must support SPI transfers
    CS: OutputPin,       // <- bound: the chip-select line must support digital output
{
    cs.set_low().ok();
    bus.write(frame)?;
    cs.set_high().ok();
    Ok(())
}
```

**Why this way:** bounding each generic parameter by the narrowest
`embedded-hal` trait it needs — rather than, say, one combined
vendor-specific "display bus" type — keeps `write_frame` usable with any
HAL's SPI and GPIO implementations mixed and matched, which is exactly
the composability `embedded-hal`'s split-by-capability trait design is
meant to buy.

### Scenario: Handling and propagating errors

Bounding a generic driver function by a trait with an associated `Error`
type lets `?` propagate that peripheral's own error type straight out of
the function, instead of the function inventing an error type of its
own.

```
fn blink_twice<P: OutputPin>(pin: &mut P) -> Result<(), P::Error> {
    pin.set_high()?; // <- P::Error only exists because of the P: OutputPin bound
    pin.set_low()?;
    pin.set_high()?;
    pin.set_low()
}
```

**Why this way:** letting each concrete pin type's own `Error` flow
through unchanged means the function makes no assumption about how a
given vendor's peripheral fails — a bare-metal HAL might use an
infallible error type, while another peripheral's bus might report real
transfer failures, and the bound-based signature accommodates both
without a conversion step.

### Scenario: Designing a public API

A generic driver function should bound its type parameter against the
narrowest `embedded-hal` trait its body actually calls — requiring a
broader trait than necessary forces every board's pin/bus type to
implement methods the function never uses.

```
// AVOID: bounds by a broader trait than the body needs
fn blink_avoid<P: OutputPin + InputPin>(pin: &mut P) -> Result<(), P::Error> {
    pin.set_high()
}

// PREFER: bound only what the body calls
fn blink<P: OutputPin>(pin: &mut P) -> Result<(), P::Error> { // <- OutputPin is the only capability this fn uses
    pin.set_high()
}
```

**Why this way:** an over-bounded driver function can only be called with
pin types that happen to implement both traits, which needlessly narrows
which boards' HALs can supply a pin — minimal bounds keep the function
usable by the widest range of concrete peripheral implementations.

---
title: "Blanket implementations"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Writing Generic & Reusable Code", "Decoupling"]
related_syntax: []
see_also: ["Trait bounds", "Traits", "Generics as type classes"]
---

## Explanation

A blanket implementation implements a trait for every type that satisfies
some bound, all at once, rather than one type at a time — for instance,
`impl<T: Display> ToString for T` gives every `Display` type `ToString`
for free, in a single `impl` block.

This is how, for example, every type implementing `Display` automatically
gets `.to_string()` in the standard library — the blanket impl covers the
entire (open-ended, unbounded) set of types satisfying `Display`, present
and future, without the standard library needing to know about any of
them individually.

Blanket impls are a powerful decoupling tool: a trait author can provide
functionality for "any type with property X" without ever enumerating
which types that includes, and library consumers get the functionality
automatically the moment their own type satisfies the bound — no explicit
opt-in `impl` required on their part. The tradeoff is that blanket impls
interact with [the orphan rule](the-orphan-rule-and-coherence.md) in
restrictive ways (only the crate defining the trait can write a blanket
impl for it), specifically to prevent two different crates from
providing conflicting blanket impls for the same trait.

## Basic usage example

```
trait Describe {
    fn describe(&self) -> String;
}

impl<T: std::fmt::Display> Describe for T { // <- one impl covers every Display type at once
    fn describe(&self) -> String {
        format!("value: {self}")
    }
}

println!("{}", 5.describe());
```

**Restriction:** only the crate that defines `Describe` may write this
blanket impl — the orphan rule forbids a downstream crate from
blanket-implementing someone else's trait for someone else's types.

## Best practices & deeper information

### Scenario: Writing generic code

A blanket impl can add a convenience method to every type satisfying a
bound at once, instead of asking each type to opt in individually —
useful when the behavior follows purely from the bound itself.

```
trait Clamp {
    fn clamp_to(self, lo: Self, hi: Self) -> Self;
}

impl<T: PartialOrd> Clamp for T { // <- one impl covers every PartialOrd type at once
    fn clamp_to(self, lo: Self, hi: Self) -> Self {
        if self < lo { lo } else if self > hi { hi } else { self }
    }
}

let level = 42.clamp_to(0, 100); // works for i32, f64, or any other PartialOrd type
```

**Why this way:** this only works because `PartialOrd` is a
[trait bound](trait-bounds.md) the compiler can check at every call site
— see [Traits](traits.md) for how `impl Trait for Type` is what makes an
implementation exist in the first place, here applied to every type
satisfying the bound rather than one named type.

## Explanation (Embedded)

The mechanism is identical under `#![no_std]` — a blanket impl is resolved
entirely at compile time, so it has no dependency on `std` or an
allocator either way. Where blanket impls earn their keep in embedded
Rust specifically is on top of `embedded-hal`'s traits: because
`embedded-hal` defines narrow, minimal traits like `OutputPin` (see
[`trait`'s embedded explanation](../../syntax/keywords/trait.md) for the
fuller embedded-hal story), a crate can add a convenience extension trait
— blanket-implemented for every type that already implements `OutputPin`
— and every vendor's pin type gets the convenience method automatically,
without the extension-trait author ever naming a single concrete chip's
pin type. The orphan rule still applies exactly as in hosted code: only
the crate defining the extension trait may write the blanket impl, so
this pattern lives in a small helper crate that defines its own trait
rather than trying to blanket-impl `embedded-hal`'s traits themselves for
foreign types.

## Basic usage example (Embedded)

```
trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

trait PulseExt {
    fn pulse(&mut self);
}

impl<P: OutputPin> PulseExt for P { // <- one impl covers every OutputPin type at once
    fn pulse(&mut self) {
        self.set_high();
        self.set_low();
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A helper crate wants to give every `embedded-hal`-compatible GPIO pin a
`pulse` convenience method without asking each vendor's HAL to implement
a new trait one type at a time.

```
trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

trait PulseExt: OutputPin {
    fn pulse_n(&mut self, times: u8) { // <- default body: only needs OutputPin's two primitives
        for _ in 0..times {
            self.set_high();
            self.set_low();
        }
    }
}

impl<P: OutputPin> PulseExt for P {} // <- blanket impl: every OutputPin implementer gets pulse_n for free

struct Gpio7;
impl OutputPin for Gpio7 {
    fn set_high(&mut self) {}
    fn set_low(&mut self) {}
}

Gpio7.pulse_n(3);
```

**Why this way:** the blanket impl covers the open-ended set of pin types
across every vendor's HAL, present and future, so `PulseExt` needs no
knowledge of which concrete pin types exist — the same decoupling
[`trait`'s embedded explanation](../../syntax/keywords/trait.md) describes
for `embedded-hal` itself, applied here to an add-on convenience method
rather than the hardware contract.

### Scenario: Designing a public API

A crate offering ergonomics on top of `embedded-hal` traits must define
its own extension trait to blanket-impl, rather than attempting to
blanket-impl `embedded-hal`'s own traits for foreign pin types.

```
// AVOID: blanket-impling a foreign trait for a foreign bound — the orphan
// rule forbids this outright, so it doesn't compile:
// impl<P: SomeVendorsPinTrait> embedded_hal::digital::OutputPin for P { ... }

// PREFER: define a new local trait and blanket-impl that instead
trait DebouncedRead {
    fn read_debounced(&self) -> bool;
}

trait InputPin {
    fn is_high(&self) -> bool;
}

impl<P: InputPin> DebouncedRead for P { // <- local trait, blanket-implemented: orphan rule is satisfied
    fn read_debounced(&self) -> bool {
        self.is_high() // a real impl would sample more than once
    }
}
```

**Why this way:** the orphan rule requires the crate writing a blanket
impl to own either the trait or the type, so an ergonomics crate built on
`embedded-hal` always introduces its own trait as the blanket-impl target
— attempting to blanket-impl `embedded-hal`'s traits directly for
"any foreign pin type" is exactly the conflict [the orphan
rule](the-orphan-rule-and-coherence.md) exists to prevent.

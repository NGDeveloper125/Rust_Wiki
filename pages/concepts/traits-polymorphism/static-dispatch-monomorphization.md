---
title: "Static dispatch & monomorphization"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Polymorphism"]
related_syntax: []
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "Generics", "Zero-cost abstractions"]
---

## Explanation

When generic code is compiled, the compiler generates a separate,
specialized copy of it for every concrete type it's actually called with
— this is monomorphization ("making one shape from many"). Calling
`largest::<i32>(...)` and `largest::<String>(...)` in the same program
produces two distinct compiled functions, each specialized to its type,
with the choice of which one to call baked in and resolved entirely at
compile time — no runtime lookup, no indirection.

This is "static" dispatch: the exact function being called is known
statically, at compile time, as opposed to
[dynamic dispatch](trait-objects-dynamic-dispatch.md) (`dyn Trait`),
where the specific implementation is chosen at runtime via a vtable
lookup. The tradeoff is binary size versus runtime cost: monomorphization
can produce larger compiled binaries (one copy per concrete type used),
but each copy runs exactly as fast as if you'd hand-written it for that
specific type — this is precisely what "zero-cost abstraction" means in
Rust's design: the abstraction (writing generic code once) costs nothing
at runtime compared to the non-abstracted equivalent (writing each
specialized version by hand yourself).

## Basic usage example

```
fn largest<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

largest(1, 2);         // <- compiler generates a largest::<i32> copy
largest("a", "b");     // <- ...and a separate largest::<&str> copy, chosen at compile time
```

## Best practices & deeper information

### Scenario: Writing generic code

A generic function called with a small, known set of concrete types is a
good candidate for static dispatch — each call site gets its own
specialized, inlinable copy with no vtable indirection.

```
fn largest<T: PartialOrd>(a: T, b: T) -> T { // <- generic: monomorphized per concrete type used
    if a > b { a } else { b }
}

largest(3, 7);       // compiler emits a largest::<i32> copy
largest(3.5, 2.1);   // ...and a separate largest::<f64> copy, both resolved at compile time
```

**Why this way:** static dispatch trades binary size for speed — no
runtime lookup, and each copy can be inlined and optimized as if
hand-written — which
[Effective Rust's item on generics vs. trait objects](https://effective-rust.com/generics.html)
recommends as the default choice; reach for
[trait objects & dynamic dispatch](trait-objects-dynamic-dispatch.md)
instead once the set of concrete types is only known at runtime or code
size becomes the binding constraint.

## Explanation (Embedded)

Static dispatch has no allocator dependency at all, and monomorphization
is exactly this page's specific angle on a tradeoff [`dyn`'s embedded
explanation](../../syntax/keywords/dyn.md) already frames at a higher
level — generics-vs-`dyn Trait` on a constrained core. The reason generic,
`embedded-hal`-bounded functions are so strongly preferred in embedded
code specifically (more so than the same preference shows up in hosted
Rust) is that monomorphization doesn't just avoid a vtable's indirect
jump — it hands the compiler a fully concrete call graph to optimize
across. A function generic over `P: OutputPin`, once monomorphized for
one concrete pin type, can often be inlined straight through: the
"abstract" call to `pin.set_high()` resolves at compile time to the one
`impl OutputPin for Pa0` that exists for that instantiation, which the
compiler can then inline into a single volatile register write — the
trait boundary can vanish from the compiled output entirely, leaving code
indistinguishable from what you'd get hand-writing the register access
with no abstraction at all. A `dyn OutputPin` call can never do this: the
concrete implementation behind the vtable is chosen at runtime, so the
compiler must leave an indirect call in place and cannot see through it to
inline anything on the other side.

The cost runs the other way in code size, and on embedded that cost is
concrete rather than abstract: every distinct concrete type a generic
function is actually instantiated with produces its own full copy of that
function's compiled code. A driver function generic over `embedded-hal`'s
`OutputPin` called with five different pin types across a board's
initialization code compiles to five separate (though each individually
optimal) copies, where the `dyn`-based equivalent would be one copy shared
by all five call sites. On a hosted target with megabytes of program
memory this rarely registers; on a chip with tens of kilobytes of flash,
instantiating a generic driver across many peripheral instances is a real
line item, and is the concrete reason some embedded codebases deliberately
reach for `dyn` (once `alloc` is available) or a hand-written enum
dispatch instead, trading away the inlining benefit specifically to cap
code size.

## Basic usage example (Embedded)

```
trait OutputPin {
    fn set_high(&mut self);
}

fn turn_on<P: OutputPin>(pin: &mut P) { // <- generic: monomorphized per concrete pin type used
    pin.set_high();
}
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A driver function that only ever configures one or two concrete pin
types in a given firmware image is a strong candidate for static
dispatch — the compiler can often inline the call chain down to a single
register write, with the trait boundary costing nothing at runtime.

```
trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

struct Pa0; // this board's only status LED pin
impl OutputPin for Pa0 {
    fn set_high(&mut self) { /* single volatile register write */ }
    fn set_low(&mut self) { /* single volatile register write */ }
}

fn blink<P: OutputPin>(pin: &mut P) { // <- monomorphized for Pa0 alone in this firmware image
    pin.set_high();
    pin.set_low();
}

blink(&mut Pa0);
```

**Why this way:** with a single concrete `P` used at the only call site,
monomorphization produces exactly one specialized copy of `blink`, which
the compiler is free to inline straight into the two register writes —
the same zero-cost-abstraction argument [Effective Rust's generics vs.
trait objects item](https://effective-rust.com/generics.html) makes
generally, sharpened here by embedded code's tighter cycle budgets, where
an indirect vtable call is a cost that's easy to measure rather than a
theoretical one.

### Scenario: Designing a public API

A HAL crate offering a generic driver function should watch how many
distinct concrete types it actually gets instantiated with across a
firmware image, since each one is a full compiled copy — instantiating
the same generic driver logic across a dozen peripheral instances can
turn a small function into a real line item in the flash budget.

```
trait OutputPin {
    fn set_high(&mut self);
}

fn set_all<P: OutputPin, const N: usize>(pins: &mut [P; N]) { // <- one copy per concrete P actually used
    for pin in pins {
        pin.set_high();
    }
}

// Called with five distinct pin types across board init compiles to
// five separate copies of set_all — each individually optimal, but each
// occupying its own flash space, unlike a single shared dyn-based version.
```

**Why this way:** the code-size side of monomorphization is the
concrete counterpart to [`dyn`'s embedded
explanation](../../syntax/keywords/dyn.md) of the same tradeoff — a HAL
author choosing generics for their inlining benefit should still budget
for one compiled copy per concrete type actually instantiated, and switch
a specific hot spot to `dyn Trait` (once `alloc` is available) or an enum
dispatch if that multiplication becomes the binding constraint rather
than execution speed.

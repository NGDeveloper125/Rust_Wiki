---
title: "virtual"
kind: keyword
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["final", "override", "dyn"]
see_also: ["final", "override", "dyn"]
---

## Explanation

`virtual` has been reserved since the 2015 edition, part of the
[Rust Reference's original reserved-keyword list](https://doc.rust-lang.org/reference/keywords.html).
It sits with [`override`](override.md) and [`final`](final.md) as
vocabulary borrowed from class-based inheritance (C++'s `virtual`
methods, or similar mechanisms in Java/C#), reserved in case Rust ever
grows an inheritance-like or virtual-dispatch mechanism distinct from
traits. No concrete proposal exists for this today — it's speculative,
same as its two neighbors.

Worth noting explicitly: Rust already has a way to get
virtual-dispatch-like behavior, and it doesn't need a `virtual` keyword
to do it. [`dyn Trait`](dyn.md) dispatches method calls through a vtable
at runtime, the same underlying mechanism C++ virtual methods use —
Rust just reaches it through ordinary trait objects rather than a
class-hierarchy keyword. That existing, working path is plausibly a big
part of why `virtual` has stayed unclaimed for over a decade: the
functional gap it might have filled is already closed by `dyn`.

Using `virtual` as an ordinary identifier is a compile error today. The
raw-identifier form `r#virtual` is legal, the same escape hatch every
reserved keyword offers.

## Usage examples

### The `virtual` reservation error and raw-identifier escape hatch

```
let virtual = 5;     // error: expected identifier, found reserved keyword `virtual`
let r#virtual = 5;   // ok: the raw-identifier form escapes the reservation
```

### Runtime polymorphism

Dispatching a call to one of several different concrete types at
runtime — the job a `virtual` method would do in a class hierarchy — is
already handled by `dyn Trait`, with no `virtual` keyword involved.

```
trait PaymentMethod {
    fn charge(&self, cents: u64);
}

struct CreditCard;
impl PaymentMethod for CreditCard {
    fn charge(&self, cents: u64) {
        println!("charging card: {cents} cents");
    }
}

struct StoreCredit;
impl PaymentMethod for StoreCredit {
    fn charge(&self, cents: u64) {
        println!("deducting store credit: {cents} cents");
    }
}

fn checkout(method: &dyn PaymentMethod, cents: u64) {
    // <- runtime dispatch through a vtable — `dyn`, not a reserved `virtual` keyword
    method.charge(cents);
}

checkout(&CreditCard, 2599);
```

`dyn Trait` already gives Rust the runtime-dispatch
behavior a `virtual` keyword would provide in a class-based language, by
dispatching through a trait object's vtable instead of a class
hierarchy — see [Trait objects & dynamic dispatch](../../concepts/traits-polymorphism/trait-objects-dynamic-dispatch.md)
for the full mechanism.

## Explanation (Embedded)

**Full support.** Keyword reservation is a lexer-level fact, identical in
`#![no_std]` and hosted Rust alike — `virtual` carries no defined meaning
on any target, so there's no embedded-specific behavior to describe. The
same is true of the working alternative already covered on the classic
page: `dyn Trait`'s vtable-based dispatch is available under
`#![no_std]` too, with the same considerations (no class hierarchy,
ordinary trait objects) covered on [`dyn`](dyn.md).

## Usage examples (Embedded)

### The `virtual` reservation, unaffected by target

```
let virtual = 5;     // error: expected identifier, found reserved keyword `virtual`, on every target
let r#virtual = 5;   // ok: the raw-identifier form escapes the reservation, on every target
```

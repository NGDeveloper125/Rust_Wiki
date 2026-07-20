---
title: "The strategy pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Object-Oriented-ish Patterns"]
related_syntax: [dyn]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "Static dispatch & monomorphization", "The command pattern", "Closures & capturing"]
---

## Explanation

The strategy pattern lets a specific behavior — a payment method, a
sort comparator, a retry policy — be swapped independently of the code
that uses it, chosen by the caller rather than hard-coded into the
consumer. Classic object-oriented form needs an abstract `Strategy`
interface, one concrete class per algorithm, and a context class holding
a reference to whichever concrete strategy was injected. Rust rarely
needs that much ceremony, because it has three lighter-weight ways to
express "pluggable behavior," and which one applies depends entirely on
*when* the strategy is chosen:

a plain function or closure parameter, bound by `Fn`/`FnMut`/`FnOnce`,
when the strategy really is just one operation and the caller already
has the right [closure](../functions-closures/closures-and-capturing.md)
in scope; a generic function or type bounded by a trait, when exactly
one strategy is picked per call site and known at compile time, so the
compiler [monomorphizes](../traits-polymorphism/static-dispatch-monomorphization.md)
it away entirely and pays no dispatch cost at all; or a `&dyn
Trait`/`Box<dyn Trait>` [trait object](../traits-polymorphism/trait-objects-dynamic-dispatch.md),
when the concrete strategy genuinely isn't known until runtime — read
from configuration, chosen by a user, or selected from a registry.

Reach for a real, multi-method `Strategy` trait — rather than a bare
closure — once a strategy needs more than one operation or its own
constructor and state: a closure covers "run this one function," but a
strategy exposing both `pay` and `refund`, say, needs a trait with two
methods. The choice between a generic bound and `dyn Trait` for that
trait is exactly the [static vs. dynamic dispatch](../traits-polymorphism/static-dispatch-monomorphization.md)
tradeoff: pay a small vtable indirection to pick the strategy at
runtime, or fix it at compile time for zero overhead but one strategy
per instantiation.

Strategy and [command](the-command-pattern.md) are close cousins and
easy to conflate: a strategy is a swappable *algorithm* invoked
repeatedly as part of some larger operation (how to price an order), a
command is a swappable *action* usually invoked once and often queued or
undone (send this specific order). Both lean on the same `&dyn
Trait`/`Box<dyn Trait>` mechanism underneath.

## Basic usage example

```
trait PaymentStrategy {
    fn pay(&self, amount_cents: u64) -> String;
}

struct CreditCard;
impl PaymentStrategy for CreditCard {
    fn pay(&self, amount_cents: u64) -> String {
        format!("Charged {amount_cents} cents to credit card")
    }
}

struct PayPal;
impl PaymentStrategy for PayPal {
    fn pay(&self, amount_cents: u64) -> String {
        format!("Charged {amount_cents} cents via PayPal")
    }
}

fn checkout(strategy: &dyn PaymentStrategy, amount_cents: u64) { // <- behavior is chosen by the caller, not hard-coded here
    println!("{}", strategy.pay(amount_cents));
}

checkout(&CreditCard, 1999);
checkout(&PayPal, 2499);
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

An online checkout doesn't know which payment method a customer picked
until the request arrives, so the concrete strategy has to be selected
and stored at runtime, not chosen at compile time.

```
trait PaymentStrategy {
    fn pay(&self, amount_cents: u64) -> String;
}

struct CreditCard;
impl PaymentStrategy for CreditCard {
    fn pay(&self, amount_cents: u64) -> String { format!("card: {amount_cents}") }
}

struct PayPal;
impl PaymentStrategy for PayPal {
    fn pay(&self, amount_cents: u64) -> String { format!("paypal: {amount_cents}") }
}

fn select_strategy(method: &str) -> Box<dyn PaymentStrategy> { // <- concrete type decided at runtime, from request data
    match method {
        "paypal" => Box::new(PayPal),
        _ => Box::new(CreditCard),
    }
}

let strategy = select_strategy("paypal");
println!("{}", strategy.pay(500));
```

**Why this way:** the payment method isn't known until the request
arrives, so `dyn Trait` is the only option that lets the concrete type be
picked after the program has already started — exactly the case the
[Rust Design Patterns' strategy entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html)
uses runtime-selected behavior for, unlike a generic parameter which
fixes the type at compile time.

### Scenario: Writing generic code

A pricing calculation always uses exactly one, compile-time-known
discount strategy per build (a "clearance" binary vs. a "regular" one),
so a generic parameter gets the same pluggability with zero dispatch
cost.

```
trait DiscountStrategy {
    fn discount(&self, price_cents: u32) -> u32;
}

struct NoDiscount;
impl DiscountStrategy for NoDiscount {
    fn discount(&self, price_cents: u32) -> u32 { price_cents }
}

struct PercentOff(u32);
impl DiscountStrategy for PercentOff {
    fn discount(&self, price_cents: u32) -> u32 {
        price_cents - (price_cents * self.0 / 100)
    }
}

fn final_price<S: DiscountStrategy>(price_cents: u32, strategy: &S) -> u32 { // <- monomorphized per strategy type: no vtable, no allocation
    strategy.discount(price_cents)
}

println!("{}", final_price(2000, &PercentOff(10)));
```

**Why this way:** when the concrete strategy is fixed at each call site,
a generic bound gives the same swappable-algorithm flexibility as
`dyn Trait` with none of the vtable indirection, since the
[Rust Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
confirms each instantiation is compiled into its own specialized,
statically-dispatched function.

## Embedded Rust Notes

**Full support.** A closure parameter, a generic bound, and `&dyn
Trait` all resolve without any heap allocation, so every shape here
works unchanged under `#![no_std]`. Only the `Box<dyn PaymentStrategy>`
variant, used when the concrete strategy is chosen at runtime, needs the
`alloc` crate — an embedded equivalent typically matches on a fixed set
of known strategies instead, the same trick shown in the "Writing
generic code" scenario, or the
[on-stack dynamic dispatch](on-stack-dynamic-dispatch.md) idiom if
runtime selection is still needed without a heap.

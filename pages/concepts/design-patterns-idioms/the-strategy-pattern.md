---
title: "The strategy pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns", "Design Patterns & Idioms", "Object-Oriented-ish Patterns"]
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

## Explanation (Embedded)

Strategy shows up in embedded code most often as a choice between
different `embedded-hal` trait implementations providing the same
capability through different means — a delay strategy backed by a
busy-wait loop versus one backed by a hardware timer's interrupt, or a
bit-banged protocol's timing driven by a fixed busy-wait versus a
calibrated cycle-counter read. All three of hosted Rust's shapes still
apply unchanged: a closure parameter, a generic bound, and `&dyn Trait`
all resolve with no heap allocation, so they work identically under
`#![no_std]`. What genuinely differs is the default preference: embedded
code reaches for the generic, monomorphized version far more often than
hosted code does, because the concrete strategy for a given build is
usually fixed at compile time anyway (this board's bit-banged I2C always
uses a busy-wait delay; that board's always uses a timer), and
monomorphization means the compiler generates one specialized,
zero-overhead function per strategy with no vtable indirection at all —
a genuinely free win on a resource-constrained target, not just a style
preference. Only `Box<dyn PaymentStrategy>`-equivalent — a strategy
genuinely selected at runtime — needs the `alloc` crate; where runtime
selection is still needed without a heap, `&dyn Trait` (the [on-stack
dynamic dispatch](on-stack-dynamic-dispatch.md) idiom) covers it.

## Basic usage example (Embedded)

```
trait DelayStrategy {
    fn delay_half_bit(&self);
}

struct BusyWaitDelay;
impl DelayStrategy for BusyWaitDelay {
    fn delay_half_bit(&self) {
        for _ in 0..1000 {} // stand-in for a calibrated busy-wait loop
    }
}

fn bit_bang_write<D: DelayStrategy>(bit: bool, delay: &D) { // <- monomorphized per delay strategy: no vtable
    let _ = bit;
    delay.delay_half_bit();
}

bit_bang_write(true, &BusyWaitDelay);
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

A bit-banged I2C driver always uses the same timing strategy for a given
board — decided by which delay source that board's build actually has
wired up — so a generic bound picks the strategy once, at compile time,
with zero runtime dispatch cost.

```
trait DelayStrategy {
    fn delay_half_bit(&self);
}

struct TimerDelay; // backed by a hardware timer peripheral
impl DelayStrategy for TimerDelay {
    fn delay_half_bit(&self) {
        // wait on a hardware timer's compare match
    }
}

struct BusyWaitDelay; // backed by a calibrated busy-wait loop
impl DelayStrategy for BusyWaitDelay {
    fn delay_half_bit(&self) {
        for _ in 0..1000 {}
    }
}

struct BitBangI2c<D: DelayStrategy> { // <- monomorphized per delay strategy: compiled into one specialized function
    delay: D,
}

impl<D: DelayStrategy> BitBangI2c<D> {
    fn write_bit(&self, bit: bool) {
        let _ = bit;
        self.delay.delay_half_bit();
    }
}

let bus = BitBangI2c { delay: TimerDelay };
bus.write_bit(true);
```

**Why this way:** when a board's timing source is fixed at build time,
a generic bound gives the same swappable-strategy structure as `dyn
Trait` with none of the vtable indirection, which is why embedded code
reaches for monomorphized generics over boxed trait objects by default —
the [Rust Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
confirms each instantiation compiles into its own specialized,
statically-dispatched function, exactly the zero-overhead property a
resource-constrained target wants.

### Scenario: Runtime polymorphism

A sensor driver needs to pick between two register-access strategies —
a direct-register-write path for one detected chip revision versus an
indirect page-select path for another — but which revision is on the
board is only known after probing an ID register at startup, so the
strategy has to be selected at runtime.

```
trait RegisterAccess {
    fn write_register(&self, addr: u8, value: u8);
}

struct DirectAccess;
impl RegisterAccess for DirectAccess {
    fn write_register(&self, addr: u8, value: u8) {
        let _ = (addr, value); // direct register write
    }
}

struct PagedAccess;
impl RegisterAccess for PagedAccess {
    fn write_register(&self, addr: u8, value: u8) {
        let _ = (addr, value); // select page, then write
    }
}

fn select_strategy(chip_id: u8) -> &'static dyn RegisterAccess { // <- decided at runtime, from a probed chip ID
    static DIRECT: DirectAccess = DirectAccess;
    static PAGED: PagedAccess = PagedAccess;
    if chip_id == 0x42 { &PAGED } else { &DIRECT }
}

let access = select_strategy(0x42);
access.write_register(0x10, 0xFF);
```

**Why this way:** the chip revision isn't known until it's probed at
runtime, so `dyn Trait` is the only option that lets the concrete
strategy be picked after the program has started — using `&'static dyn
RegisterAccess` over `static` values instead of `Box::new` keeps the
runtime-selection benefit from the
[Rust Design Patterns' strategy entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html)
without needing `alloc` at all, per the
[on-stack dynamic dispatch](on-stack-dynamic-dispatch.md) idiom.

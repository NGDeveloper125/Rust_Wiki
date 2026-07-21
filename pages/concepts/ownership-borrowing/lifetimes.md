---
title: "Lifetimes"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Lifetime Management", "Unique to Rust"]
related_syntax: ["'a", "&"]
see_also: ["The borrow checker", "Lifetime elision", "Borrowing (shared references)"]
---

## Explanation

A lifetime describes how long a reference remains valid — specifically,
that it cannot outlive the value it points to. Every reference in Rust
has a lifetime, whether or not it's written out explicitly; naming it
(`'a`) becomes necessary only when the compiler can't work out on its own
that a function's inputs and outputs relate correctly.

The concept lifetimes exist to express is straightforward: `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`
says "the reference this function returns lives no longer than the
shorter-lived of the two inputs" — a real constraint on the function's
contract, not a formality. Without that annotation, the compiler would
have no way to verify the caller isn't left holding a dangling reference
after the inputs it borrowed from go out of scope.

Lifetimes are a **compile-time-only** concept — they exist purely to let
the borrow checker prove reference validity ahead of time, and are erased
entirely before the program runs (there is no runtime lifetime tracking,
no reference counting implied by writing `'a`). This is precisely why
Rust can guarantee no dangling references with zero runtime cost: the
proof happens once, at compile time, instead of via a runtime check
(garbage collection, reference counting) every single language with
manual memory management otherwise needs.

## Basic usage example

```
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { // <- ties the output's lifetime to both inputs
    if x.len() > y.len() { x } else { y }
}

let s1 = String::from("long string");
let result;
{
    let s2 = String::from("short");
    result = longest(&s1, &s2);
    println!("{result}"); // must be used while s2 is still alive
}
```

**Restriction:** the returned reference can't outlive the shorter-lived
input — using `result` after `s2` goes out of scope would fail to
compile, since `'a` is bound by the shorter of the two borrows.

## Best practices & deeper information

### Scenario: Designing a public API

A `Parser`'s methods should carry an explicit lifetime parameter only
where the elision rules genuinely can't infer one — anywhere they can,
writing `'a` out by hand adds noise without adding a constraint.

```
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    // AVOID: writing out a lifetime the elision rules already infer for free
    // fn peek<'b>(&'b self) -> Option<&'b str> { self.input.get(self.pos..) }

    fn peek(&self) -> Option<&str> { // <- PREFER: elided; output borrows from `&self` automatically
        self.input.get(self.pos..)
    }
}
```

**Why this way:** an explicit lifetime that elision would have inferred
anyway doesn't express anything the compiler didn't already know — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/) favor the
simplest signature that expresses the real contract, which keeps a
type's explicit lifetime parameters meaningful on the cases where they
truly are load-bearing (see [Lifetime elision](lifetime-elision.md)).

### Scenario: Sharing data with multiple references

A function borrowing from two differently-lived inputs needs its
lifetime annotation to be what lets the compiler catch a caller trying to
use the result after the shorter-lived input is gone.

```
fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str { // <- ties the result to whichever input's borrow ends first
    if a.len() < b.len() { a } else { b }
}

let long_lived = String::from("configuration");
let result;
{
    let short_lived = String::from("cfg");
    result = shorter(&long_lived, &short_lived);
    println!("{result}"); // must run while short_lived is still alive
}
// using `result` here would fail to compile: it may borrow from short_lived, now dropped
```

**Why this way:** naming the shared lifetime `'a` across both parameters
and the return type is what lets the borrow checker reject uses of
`result` after the shorter-lived input is gone — without it, the compiler
would otherwise have to assume the return value could outlive either
input, which the
[Rust Book](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
covers as the core case explicit lifetimes exist for.

## Explanation (Embedded)

Mechanically, lifetimes mean exactly what the classic explanation says on
a bare-metal target too — a compile-time-only proof of reference validity,
fully erased before codegen, with zero runtime cost on any target. The
syntax-level mechanics — how to write `'a` on a struct, when `'a: 'b` is
needed, what `'static` requires — are covered from the token's own angle
by ['a (named lifetime)](../../syntax/lifetimes/named-lifetime.md) and
[`'static`](../../syntax/lifetimes/static-lifetime.md); this page's job is
the design-level question those pages don't ask: *why* does embedded code
end up caring about lifetimes more, and more consequentially, than a lot
of hosted application code does?

Two reasons. First, `embedded-hal`-style driver design is built around
*borrowing* a peripheral rather than owning it, specifically so one
physical bus or pin can be shared between several drivers that each hold
it only while actively using it — and the moment a struct borrows rather
than owns, it needs a named lifetime parameter tying its own validity to
the thing it borrowed, exactly as
['a (named lifetime)](../../syntax/lifetimes/named-lifetime.md) shows
mechanically. Choosing borrow-with-a-lifetime versus own-outright for a
driver struct is a genuine, load-bearing embedded API design decision, not
just a syntax choice: owning outright is simpler to write and reason
about, but locks the peripheral to that one driver for good; borrowing
lets several drivers take turns with the same bus, at the cost of a
lifetime parameter every user of the struct now has to thread through
their own code. Second, an interrupt handler has no caller's stack frame
to bound it — it can fire at any point after the vector table is
installed, independent of whatever's running in `main` — so any data it
touches can't carry an ordinary borrowed lifetime tied to a `main`-local
variable at all; it has to be promoted to `'static` storage, almost always
a `static` guarded by `critical_section::Mutex<Cell<_>>`/`Mutex<RefCell<_>>`
(see [`'static`](../../syntax/lifetimes/static-lifetime.md) and
[Interior mutability](interior-mutability.md)). Recognizing *which* of the
two situations a given piece of state is in — genuinely scoped and
borrowable, versus needing to survive into an unpredictable future
interrupt — is itself a design judgment lifetimes force you to make
explicit, in a way a language without them would let you paper over until
it broke at runtime.

## Basic usage example (Embedded)

```
struct SpiReading<'a> {
    raw: &'a [u8], // <- borrowed: this reading doesn't own the buffer it was read into
}

fn latest<'a>(buf: &'a [u8]) -> SpiReading<'a> {
    SpiReading { raw: buf }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

Deciding whether an accelerometer driver should borrow the SPI bus for one
reading, borrow it for the driver's whole lifetime, or own the bus
outright — three different lifetime shapes for the same underlying task,
each with a real tradeoff.

```
// (a) borrow only for the call: elision handles this, no named lifetime needed at all
fn read_once(spi: &mut impl embedded_hal::spi::SpiBus) -> i16 { /* ... */ 0 }

// (b) borrow for the driver's whole life: needs an explicit lifetime parameter
struct Accelerometer<'a, SPI> {
    spi: &'a mut SPI, // <- other code can still use `spi` once this driver is dropped
}

// (c) own the bus outright: no lifetime parameter, but the bus is locked to this driver for good
struct OwningAccelerometer<SPI> {
    spi: SPI,
}
```

**Why this way:** shape (a) is right when nothing else needs the bus
between readings; shape (b) is right when the bus is shared with other
peripherals the rest of the program still needs to reach; shape (c) is
right once a driver is the bus's only real user — the named lifetime in
(b) is the compiler-enforced promise that this driver's borrow really
does end, letting the bus be reused elsewhere, which
['a (named lifetime)'s embedded section](../../syntax/lifetimes/named-lifetime.md)
covers from the mechanics side.

### Scenario: Sharing state across threads

A UART receive interrupt needs to hand its last error code to the main
loop — unlike the accelerometer above, this data can't be borrowed from
any `main`-local variable at all, because the interrupt has no
relationship to whatever `main` happens to be running when it fires.

```
use core::cell::Cell;
use critical_section::Mutex;

static LAST_UART_ERROR: Mutex<Cell<Option<u8>>> = Mutex::new(Cell::new(None));
// <- 'static storage, not a borrowed &'a field: the interrupt has no caller stack frame to tie a lifetime to

#[interrupt]
fn USART1() {
    critical_section::with(|cs| LAST_UART_ERROR.borrow(cs).set(Some(read_error_flags())));
}
```

**Why this way:** a driver borrowing a bus (previous scenario) and an
interrupt-shared error flag look similar at a glance — both involve a
lifetime-shaped decision — but they resolve oppositely: the driver's
borrow genuinely ends, so a named `'a` is the right, checked promise; the
interrupt's data can't be tied to any stack frame at all, so it has to be
`'static` instead. Conflating the two is a common early mistake, and
[`'static`'s embedded section](../../syntax/lifetimes/static-lifetime.md)
covers why interrupt-shared data specifically needs the second shape.

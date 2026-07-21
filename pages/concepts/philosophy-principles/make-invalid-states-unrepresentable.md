---
title: "\"Make invalid states unrepresentable\""
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Declarative / Metaprogramming", "Designing Robust Data Models", "Unique to Rust"]
related_syntax: []
see_also: ["Enums (algebraic data types)", "Option<T>", "The newtype pattern", "Const generics", "match expressions"]
---

## Explanation

The principle is simple to state and unusually consequential in practice:
model your data so that a value which shouldn't exist literally cannot be
constructed, rather than modeling it loosely and then guarding against the
bad cases with scattered runtime checks. The phrasing is often traced to
Yaron Minsky's writing on OCaml ("make illegal states unrepresentable"),
but it became something close to a rallying idea in Rust's own community
specifically because Rust's type system gives it unusually sharp teeth
compared to most mainstream languages — this page's job is to name that
principle and survey where it shows up across the language; the mechanics
of any one tool that embodies it live on that tool's own page.

The core mechanism is the enum as a genuine sum type — see
[Enums (algebraic data types)](../types-data-modeling/enums-algebraic-data-types.md)
for the full treatment. A value of an enum type is always exactly one of
its variants, never a partial mix of two, and an exhaustive `match` forces
every variant to be handled, so a case added later surfaces as a compile
error everywhere it matters instead of a silent gap.
[`Option<T>`](../error-handling/option.md) is the smallest possible
instance of the idea: absence becomes a distinct, checked case in the type
itself rather than a null value that can silently stand in for a real one
anywhere a real one was expected. [The newtype pattern](../types-data-modeling/the-newtype-pattern.md)
applies the same discipline to primitives: wrapping a raw `u64` as
`UserId` rather than leaving it a bare `u64` makes swapping it with an
`OrderId` a compile error instead of a bug that only shows up in
production, and pairing a newtype with a validating constructor — the
"parse, don't validate" idiom — means that once a value exists as that
type at all, its invariant has already been checked; nothing downstream
has to re-verify it.

The same idea keeps extending further into the type system.
[Const generics](../types-data-modeling/const-generics.md) let a value —
an array's length, most commonly — become part of a type itself, so a
capacity mismatch between two differently-sized buffers is a compile
error rather than a bounds check that might fire at runtime. Encoding a
protocol's legal states as distinct types, so a method is simply not
callable until a prerequisite step has produced the type that carries it
(a "typestate"-flavored API), pushes the same principle all the way to
sequences of operations, not just single values — the invalid *order* of
operations becomes unrepresentable, not just the invalid data.

None of this comes for free, and it's worth being honest about the cost.
Designing a type that makes an invalid state genuinely unconstructable is
real, upfront work — usually more types, constructors that return
`Result` instead of a bare struct literal, and an API surface that asks
callers to go through a validating entry point instead of just filling in
fields. It also doesn't cover everything: invariants that depend on the
combined state of two different values ("this discount code is only valid
for this specific order") are still ordinarily a runtime check, not
something the type system alone can express, and no amount of internal
type design prevents malformed data from arriving at a program's boundary
in the first place — a request body still has to be parsed once. What the
principle buys is narrower and still very real: once a value has been
constructed as a given type, nothing later in the program can reintroduce
the invalid state that type was designed to rule out.

The plainest way to feel the difference is a direct contrast: a `status:
String` field can hold `"pending"`, `"Pending"`, or a plain typo like
`"pendnig"`, and every single place that reads it has to defensively
re-check which of those it actually got, forever. A `status: OrderStatus`
field, where `OrderStatus` is an enum of exactly the legal states, can
only ever be one of those states — checked once, at construction, and
enforced everywhere else by the compiler rather than by the discipline of
whoever wrote the next function that touches it.

## Basic usage example

```
struct LegacyOrder { status: String } // <- "pending", "Pending", "pendnig"... nothing here stops any of these

enum OrderStatus { Pending, Shipped, Delivered }
struct Order { status: OrderStatus } // <- status can only ever be one of the three variants, checked at construction
```

## Best practices & deeper information

### Scenario: Validating input

A signup form's email field should be impossible to hold an unvalidated
string once it's past the point of entry — wrapping it in a newtype with
a private field and a validating constructor means every later use of the
value can trust it's already correct.

```
struct EmailAddress(String); // <- private field: constructible only through parse()

impl EmailAddress {
    fn parse(raw: &str) -> Result<Self, String> {
        if raw.contains('@') && !raw.starts_with('@') && !raw.ends_with('@') {
            Ok(EmailAddress(raw.to_string()))
        } else {
            Err(format!("'{raw}' is not a valid email address"))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

fn send_confirmation(to: &EmailAddress) { // <- by the time a value reaches here, it's already validated
    println!("sending confirmation to {}", to.as_str());
}

let email = EmailAddress::parse("ada@example.com").expect("valid address");
send_confirmation(&email);
```

**Why this way:** keeping the field private so `EmailAddress` can only be
built through `parse` means validation happens exactly once, at
construction, instead of at every call site that touches the value — an
application of "parse, don't validate" as covered by
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/).

### Scenario: Designing a public API

A public function that sums payments should make mixing currencies a
compile error, not a runtime bug — encoding the currency as part of the
type itself, rather than as a same-shaped numeric field plus a separate
string, closes off the invalid combination entirely.

```
pub enum Money { // <- currency is part of the type: a Usd value can never be silently mixed with Eur
    Usd(u64), // cents
    Eur(u64), // cents
}

pub fn total_usd(payments: &[Money]) -> Result<u64, &'static str> {
    let mut total = 0u64;
    for payment in payments {
        match payment {
            Money::Usd(cents) => total += cents,
            Money::Eur(_) => return Err("cannot sum USD and EUR without an explicit conversion"), // <- forced, not forgotten
        }
    }
    Ok(total)
}
```

**Why this way:** [Effective Rust](https://effective-rust.com/) treats
encoding a domain invariant like currency directly into the type, rather
than as two same-shaped fields a caller could mismatch, as central to
designing a robust public API — the invalid combination is ruled out by
the type checker instead of relying on every caller to remember a rule
that lives only in documentation.

## Explanation (Embedded)

This principle earns its keep especially hard in embedded code, because
the alternative it replaces is everywhere on a microcontroller: a raw
register is, at bottom, just a `u8`/`u16`/`u32` bit pattern, and plenty of
bit patterns a register's width can physically hold are reserved,
undefined, or simply invalid for that peripheral. A UART's baud-rate
divisor field, a sensor's power-mode select bits, a GPIO's mode-select
bits — each has a small set of documented, legal values sitting inside a
much larger space of bit patterns the hardware happens to allow you to
write. Modeling that field as a bare integer means every read from the
register, and every value about to be written to it, is a fresh
opportunity to hold or send a bit pattern the datasheet never defined.
Modeling it instead as an enum with exactly the legal variants — and
converting between the enum and the raw register value at exactly one
point, via `TryFrom`/`From` — means the invalid bit patterns simply have
no corresponding enum value to exist as; a variable of that enum type is
always one of the states the hardware actually documents.

HAL crates push the same idea one step further with the "typestate"
pattern: a GPIO pin's current mode (input, output, alternate function)
becomes part of the pin's *type*, not a runtime flag checked inside each
method. `set_high()` simply doesn't exist as a callable method on a pin
typed as an input — the invalid operation is rejected at compile time by
the same mechanism that rejects calling a `Shipped`-only method on an
`OrderStatus::Pending` order, and unlike a runtime "is this pin configured
as output?" check, it costs nothing on hardware where every cycle and
every byte of flash is budgeted.

## Basic usage example (Embedded)

```
#[repr(u8)]
enum BaudRate { // <- only the documented, legal divisor codes exist as values of this type
    Baud9600 = 0x04,
    Baud19200 = 0x08,
    Baud115200 = 0x40,
}

impl TryFrom<u8> for BaudRate {
    type Error = u8; // the raw, unrecognized register value

    fn try_from(raw: u8) -> Result<Self, u8> {
        match raw {
            0x04 => Ok(BaudRate::Baud9600),
            0x08 => Ok(BaudRate::Baud19200),
            0x40 => Ok(BaudRate::Baud115200),
            other => Err(other), // <- every other bit pattern the register could hold is rejected here, once
        }
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A UART driver reading its baud-rate register back after configuring it
should trust the value it gets, rather than re-checking "is this one of
the valid codes?" at every later call site — converting the raw register
byte into the enum once, at the boundary, is what makes that trust sound.

```
struct UartRegisters { baud_select: u8 } // stand-in for a real MMIO register read

fn configured_baud_rate(regs: &UartRegisters) -> Result<BaudRate, &'static str> {
    BaudRate::try_from(regs.baud_select) // <- the one place a bad bit pattern can still surface, as an Err
        .map_err(|bits| { let _ = bits; "register holds an undefined baud-rate code" })
}

fn set_baud_rate(regs: &mut UartRegisters, rate: BaudRate) {
    regs.baud_select = rate as u8; // <- only ever one of the three legal codes gets written back
}
```

**Why this way:** a `baud_select: u8` field could hold any of 256 bit
patterns, only three of which the hardware documents as meaningful;
converting through `BaudRate` at the register boundary means every other
piece of driver code that reads `configured_baud_rate()`'s result works
with a value the type system already guarantees is one of the three legal
states, rather than re-validating a raw byte on every use — the same
"parse, don't validate" discipline from
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/) that
the classic Explanation applies to `EmailAddress`, applied here to a
hardware register instead of user input.

### Scenario: Designing a public API

A GPIO HAL should make `set_high()` uncallable on a pin still configured
as an input, rather than making it a runtime check the caller can forget
— the typestate pattern moves that invariant into the type itself.

```
struct Input;
struct Output;

struct Pin<MODE> { number: u8, _mode: core::marker::PhantomData<MODE> }

impl Pin<Input> {
    fn into_output(self) -> Pin<Output> { // <- consumes the Input-typed pin, returns an Output-typed one
        Pin { number: self.number, _mode: core::marker::PhantomData }
    }
}

impl Pin<Output> {
    fn set_high(&mut self) { /* write the peripheral's BSRR register */ }
}

let pin: Pin<Input> = Pin { number: 5, _mode: core::marker::PhantomData };
let mut pin = pin.into_output(); // <- must convert before this compiles
pin.set_high();
```

**Why this way:** encoding the pin's configuration in its type means
calling `set_high()` on a pin still wired as an input is a compile error
at the call site, not a runtime `if self.mode != Output` check that has to
execute on every single toggle of a pin that hardware access can't afford
to spend cycles re-verifying — this is the [embedded-hal](https://docs.rs/embedded-hal/)
ecosystem's standard answer to making a peripheral's invalid configuration
states unrepresentable.

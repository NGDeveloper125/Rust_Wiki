---
title: "Zero-sized types & PhantomData"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models"]
related_syntax: []
see_also: ["Unit structs", "\"Make invalid states unrepresentable\""]
---

## Explanation

A zero-sized type occupies no memory at runtime at all — `size_of::<T>() == 0`
— while still existing fully as a type at compile time. The unit type
`()`, unit structs (`struct Marker;`), and a single-variant field-less
enum are the most common naturally-occurring examples (a *multi*-variant
field-less enum like `Ordering` isn't zero-sized — it still needs a
discriminant byte): the compiler doesn't need to store anything to
represent a value that carries no data, since there's only ever one
possible value of that type.

`PhantomData<T>` is a special zero-sized type used to tell the compiler
"pretend this struct owns/relates to a `T`" without actually storing a
`T` anywhere in the struct — needed when a generic parameter is used only
in a way the compiler can't see directly (for example, in raw pointers
inside an `unsafe` implementation), so that lifetime checking, variance,
and drop-check analysis still treat the struct as if it genuinely
contained a `T`.

Both are examples of a broader theme: using the type system to carry
information that has real compile-time meaning but zero runtime cost —
the compiler tracks and enforces it, and none of it survives into the
compiled binary as actual bytes.

## Basic usage example

```
use std::marker::PhantomData;

struct Typed<T> {
    value: u32,
    _marker: PhantomData<T>, // <- no T is stored, but the compiler treats this as "owning" a T
}

let x: Typed<f64> = Typed { value: 1, _marker: PhantomData };
```

**Restriction:** `PhantomData<T>` contributes zero bytes to the struct's
size, but it isn't purely decorative — it still affects variance and
drop-check analysis as if a real `T` were stored, which can change
whether certain lifetime or drop patterns compile.

## Best practices & deeper information

### Scenario: Writing generic code

A typestate-style marker parameter lets a generic type encode which
state a value is in — so operations invalid for that state simply don't
exist as callable methods — at zero bytes of runtime cost.

```
use std::marker::PhantomData;

struct Open;   // <- zero-sized marker types
struct Closed;

struct Connection<State> {
    socket_fd: i32,
    _state: PhantomData<State>, // <- costs 0 bytes; only tracked at compile time
}

impl Connection<Closed> {
    fn open(self) -> Connection<Open> { // <- consumes a Closed connection, returns an Open one
        Connection { socket_fd: self.socket_fd, _state: PhantomData }
    }
}

impl Connection<Open> {
    fn send(&self, data: &[u8]) { /* ... */ }
    // send() simply doesn't exist on Connection<Closed> -- calling it on a
    // closed connection is a compile error, not a runtime panic
}
```

**Why this way:** this typestate pattern, covered in the
[Embedded Rust Book's Typestate Programming chapter](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html),
moves a whole category of "used it in the wrong order" bugs from a
runtime check to a compile error, and `PhantomData<State>` is what makes
the state parameter free — it contributes nothing to `Connection`'s
runtime size.

## Explanation (Embedded)

`PhantomData<Mode>` is the mechanism that makes the `embedded-hal`
typestate pattern possible without paying for it. A generic peripheral
type like `Pin<MODE>` needs to carry `MODE` as a type parameter so the
compiler can accept or reject method calls based on which mode the pin is
currently in — but the pin's actual runtime representation (a port and
pin number, say) doesn't need to store anything about `MODE` at all; the
mode only exists to be checked at compile time. Without `PhantomData<MODE>`
sitting in the struct, using `MODE` purely as a type parameter with no
field of that type would be a compile error (an unused generic
parameter), so `PhantomData<MODE>` is what lets the struct legally "hold"
a `MODE` it never actually stores — contributing exactly zero bytes to
`size_of::<Pin<MODE>>()` regardless of how many mode types exist. This is
what lets a HAL crate offer a `Pin<Input>`, a `Pin<Output>`, and a
`Pin<Analog>` that are all the same size in memory as a pin with no mode
tracking at all, while still making it a compile error to call an
output-only method on a `Pin<Input>`. On a target where every byte of RAM
is accounted for, "type-safety that costs zero bytes" isn't a
nice-to-have — it's the only way the typestate pattern is viable at all.

## Basic usage example (Embedded)

```
use core::marker::PhantomData;

struct Input;
struct Output;

struct Pin<MODE> {
    pin_number: u8,
    _mode: PhantomData<MODE>, // <- zero bytes; MODE is tracked only by the compiler
}

let led: Pin<Output> = Pin { pin_number: 5, _mode: PhantomData };
```

## Best practices & deeper information (Embedded)

### Scenario: Writing generic code

Converting a pin from one mode to another consumes the old typed value
and returns a new one carrying a different `PhantomData` marker — the
conversion is real at compile time and free at runtime.

```
use core::marker::PhantomData;

struct Input;
struct Output;

struct Pin<MODE> {
    pin_number: u8,
    _mode: PhantomData<MODE>,
}

impl Pin<Input> {
    fn into_output(self) -> Pin<Output> { // <- consumes an Input pin, returns an Output pin
        Pin { pin_number: self.pin_number, _mode: PhantomData }
    }
}

impl Pin<Output> {
    fn set_high(&mut self) { /* write to the ODR register here */ }
}
```

**Why this way:** `into_output` takes `self` by value, so the old
`Pin<Input>` can't be used again after conversion — combined with
`PhantomData` costing zero bytes, the whole mode-tracking mechanism adds
compile-time-only safety with no runtime footprint, exactly the trade the
[Rust Book's generics coverage](https://doc.rust-lang.org/book/ch10-01-syntax.html)
describes for monomorphized generic code in general.

### Scenario: Designing a public API

Restricting a method to only the mode where it's valid — an analog read
that only exists on `Pin<Analog>` — makes calling it on the wrong mode a
compile error instead of a runtime error code the caller has to remember
to check.

```
use core::marker::PhantomData;

struct Analog;
struct Digital;

struct Pin<MODE> {
    pin_number: u8,
    _mode: PhantomData<MODE>,
}

impl Pin<Analog> {
    fn read_raw(&self) -> u16 { 0 /* real code reads the ADC register here */ }
    // read_raw() simply doesn't exist on Pin<Digital> -- the method isn't
    // callable at all, rather than returning an error code at runtime
}
```

**Why this way:** an ADC read on a pin that isn't wired to the analog
peripheral is a class of mistake worth catching before the code ever
runs on hardware — making the method not exist for the wrong mode, via a
zero-cost `PhantomData` marker, is strictly stronger than a runtime
`Result` the caller could forget to check, and costs nothing to enforce.

### Scenario: Implementing traits

An `embedded-hal` trait like `OutputPin` is typically implemented only
for the mode where it's physically valid, so a generic function bounded
by that trait only ever compiles against a correctly-configured pin.

```
use core::marker::PhantomData;

struct Output;
struct Input;

struct Pin<MODE> {
    pin_number: u8,
    _mode: PhantomData<MODE>,
}

trait OutputPin {
    fn set_high(&mut self);
}

impl OutputPin for Pin<Output> { // <- only Pin<Output> gets this impl; Pin<Input> does not
    fn set_high(&mut self) { /* write to the ODR register here */ }
}

fn blink<P: OutputPin>(pin: &mut P) { // <- generic over any type that implements OutputPin
    pin.set_high();
}
```

**Why this way:** implementing the trait only for `Pin<Output>` means
`blink` simply won't compile if called with a `Pin<Input>` — the
trait-bound generic function is checked entirely at compile time, and the
`PhantomData<MODE>` marker that makes `Output` and `Input` distinct types
costs nothing in the compiled program.

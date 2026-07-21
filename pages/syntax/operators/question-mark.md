---
title: "?"
kind: operator
embedded_support: full
groups: ["Error Handling", "Traits & Polymorphism"]
related_concepts: ["The ? operator (concept angle)", "Trait bounds"]
related_syntax: ["fn", "return"]
see_also: ["fn", "return"]
---

## Explanation

`?` has two unrelated meanings, separated entirely by where it appears:

1. **Postfix on an expression: error propagation.** `expr?`, written
   directly after a `Result`- or `Option`-producing expression, unwraps
   the success value and lets execution continue, or exits the enclosing
   function immediately with the failure. This is by far the more common
   use, and *when and why* to reach for it — as opposed to a `match` or a
   combinator — is covered on
   [The ? operator (concept angle)](../../concepts/error-handling/the-question-mark-operator.md).
   The syntax angle here is the exact desugaring and where the token is
   legal to write at all.
2. **Prefix inside a trait bound: `?Sized`.** Immediately before a trait
   name in bound position (`T: ?Sized`), `?` relaxes an implicit default
   bound rather than adding a requirement — see
   [Trait bounds](../../concepts/traits-polymorphism/trait-bounds.md) for
   why every generic type parameter carries an implicit bound to begin
   with.

**Error propagation.** On a `Result<T, E>` expression, `expr?` is
equivalent to matching it: an `Ok(value)` evaluates the whole `expr?` to
`value`; an `Err(error)` immediately returns `Err(From::from(error))` from
the enclosing function — the `From::from` conversion runs even when the
error type doesn't change, since `From<T> for T` is a blanket impl. On an
`Option<T>` expression, the shape is the same minus the conversion:
`Some(value)` evaluates to `value`, `None` returns `None` from the
enclosing function unchanged. Because `?` returns from the *innermost
enclosing function or closure body*, not from whatever `match` or block
it happens to sit inside, its legality is decided entirely by that
function's declared return type: a `Result` operand requires an enclosing
return type of `Result<_, F>` with `E: Into<F>`; an `Option` operand
requires `Option<_>`. Mixing families (a `Result` operand inside a
function returning `Option`, or vice versa) is a compile error, and so is
using `?` at all inside a function whose return type implements neither
shape — most commonly `()` — with rustc reporting that "the `?` operator
can only be used in a function that returns `Result` or `Option` (or
another type that implements `Try`)." `fn main` and `#[test]` functions
are ordinary in this respect; they simply need to be declared
`-> Result<(), E>` for a top-level `?` to be legal inside them.

**`?Sized`.** Every generic type parameter carries an implicit `Sized`
bound unless relaxed — `fn f<T>(x: T)` really means `fn f<T: Sized>(x: T)`,
because a plain `T` passed by value needs a compile-time-known size.
Writing `T: ?Sized` (inline, or in a `where` clause) removes that one
implicit bound, letting the compiler instantiate `T` with an unsized type
such as `str`, `[U]`, or `dyn Trait`. `?Sized` is a relaxation, not a bound
in the usual sense, and `Sized` is currently the *only* trait `?` may
relax this way — `?SomeOtherTrait` is not legal syntax. Because relaxing
`Sized` means `T`'s size becomes unknown at compile time, a `?Sized`
parameter almost always appears behind a pointer (`&T`, `Box<T>`, `Rc<T>`)
rather than taken or returned by value, since only pointer-sized things
can be passed around without knowing the pointee's own size.

## Usage examples

### Propagating a parse error out of a function

```
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    let port: u16 = raw.trim().parse()?; // <- unwraps to u16 on Ok, returns Err early otherwise
    Ok(port)
}
```

**Restriction:** `?` is only legal here because `parse_port` returns
`Result`; moving the same line into a function returning `()` is a
compile error, not a warning — see Explanation above.

### Handling and propagating errors

Loading a calibration offset from disk can fail two independent ways —
the file might be missing, or its contents might not parse — and `?`
handles both without a `match` at either step, but only because the
function's return type is `Result`-shaped throughout.

```
use std::fs;

fn load_calibration_offset(path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?; // <- legal: enclosing fn returns Result
    let offset: f64 = contents.trim().parse()?; // <- same fn, same Result family, so `?` applies again
    Ok(offset)
}

// AVOID: `?` cannot appear in a fn whose return type isn't Result/Option-shaped —
// this fails to compile with "the `?` operator can only be used in a function
// that returns `Result` or `Option`":
//
// fn log_calibration_offset(path: &str) {
//     let contents = fs::read_to_string(path)?;
//     println!("offset file: {contents}");
// }
```

`?`'s legality is a purely syntactic property of the
enclosing function's declared return type, not of any particular branch
it's written inside, per the
[Rust Reference's operator-expression chapter](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator) —
changing `log_calibration_offset`'s signature is the only fix, commenting
around the call is not.

### Writing generic code

A logging helper that accepts anything displayable shouldn't force
callers to hand it an already-`Sized` value — bounding its parameter with
`?Sized` lets it take a bare `str` or a `dyn` trait object behind a
reference, not just references to ordinarily-sized types.

```
use std::fmt::Display;

fn log_reading<T: ?Sized + Display>(label: &T) {
    // <- `?Sized` relaxes the implicit `Sized` bound; without it, the call below wouldn't compile
    println!("reading: {label}");
}

log_reading("sensor-07"); // T = str, unsized
log_reading(&42.5);       // T = f64, ordinarily sized — ?Sized still permits this too

trait Sensor {
    fn read(&self) -> f64;
}

struct Thermometer;

impl Sensor for Thermometer {
    fn read(&self) -> f64 { 21.5 }
}

fn describe<T: ?Sized + Sensor>(sensor: &T) -> f64 { // <- also accepts `dyn Sensor`, which is unsized
    sensor.read()
}

let boxed: Box<dyn Sensor> = Box::new(Thermometer);
describe(&*boxed);
```

Leaving the implicit `Sized` bound in place would
reject both `log_reading("sensor-07")` and `describe(&*boxed)` at the
type-check stage, since `str` and `dyn Sensor` have no compile-time-known
size — `?Sized` is how a generic function opts into accepting them, per
the [Rust Reference's dynamically sized types chapter](https://doc.rust-lang.org/reference/dynamically-sized-types.html).

## Explanation (Embedded)

`?` works exactly the same, unchanged, under `#![no_std]` — it desugars
to a plain `match` plus `From::from` over `core::result::Result` /
`core::option::Option`, both core-language types with no allocator or
runtime involvement, so nothing about `#![no_std]` restricts it. This
makes `?` quietly valuable for embedded driver code specifically: an I2C
or SPI transaction is naturally a chain of several fallible steps — write
a command byte, read back a response, write another byte — and `?` lets
each step propagate its failure without a `match` at every line, exactly
the pattern from the classic Explanation above. One thing worth being
explicit about: `?`'s `From::from` conversion needs the enclosing
function's error type to implement `From<E>` for whatever error `E` each
step produces, and that has nothing to do with `std::error::Error` —
which isn't available without `std` at all (`core::error::Error` now
exists separately) — no more than hosted code strictly needs it either.
Embedded driver crates typically define their own small, `#[derive(Debug)]`
no_std error enum with a `From` impl per underlying error source (an I2C
error, a SPI error, a timeout), and `?` chains through that enum exactly
as it would through any hosted `Result` type.

## Usage examples (Embedded)

### Propagating I2C transaction errors through a custom no_std error type

```
use embedded_hal::i2c::{I2c, Error as I2cErrorTrait};

#[derive(Debug)]
enum SensorError {
    Bus,
    UnexpectedId(u8),
}

impl<E: I2cErrorTrait> From<E> for SensorError {
    fn from(_error: E) -> Self {
        SensorError::Bus // <- collapses any bus-level error into one variant
    }
}

const WHO_AM_I_REG: u8 = 0x0F;
const EXPECTED_ID: u8 = 0x68;

fn read_who_am_i(i2c: &mut impl I2c, addr: u8) -> Result<u8, SensorError> {
    let mut buf = [0u8; 1];
    i2c.write_read(addr, &[WHO_AM_I_REG], &mut buf)?; // <- `?` converts any I2C error into SensorError via the impl above
    if buf[0] != EXPECTED_ID {
        return Err(SensorError::UnexpectedId(buf[0]));
    }
    Ok(buf[0])
}
```

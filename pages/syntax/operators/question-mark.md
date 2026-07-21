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

## Basic usage example

```
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    let port: u16 = raw.trim().parse()?; // <- unwraps to u16 on Ok, returns Err early otherwise
    Ok(port)
}
```

**Restriction:** `?` is only legal here because `parse_port` returns
`Result`; moving the same line into a function returning `()` is a
compile error, not a warning — see Explanation above.

## Best practices & deeper information

### Scenario: Handling and propagating errors

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

**Why this way:** `?`'s legality is a purely syntactic property of the
enclosing function's declared return type, not of any particular branch
it's written inside, per the
[Rust Reference's operator-expression chapter](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator) —
changing `log_calibration_offset`'s signature is the only fix, commenting
around the call is not.

### Scenario: Writing generic code

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

**Why this way:** leaving the implicit `Sized` bound in place would
reject both `log_reading("sensor-07")` and `describe(&*boxed)` at the
type-check stage, since `str` and `dyn Sensor` have no compile-time-known
size — `?Sized` is how a generic function opts into accepting them, per
the [Rust Reference's dynamically sized types chapter](https://doc.rust-lang.org/reference/dynamically-sized-types.html).

## Embedded Rust Notes

**Full support.** Both meanings are core-language. `expr?` desugars to a
plain `match` plus `From::from` over `core::result::Result` /
`core::option::Option`, so it works identically under `#![no_std]` with
no allocator or runtime involved. `?Sized` is a purely compile-time
relaxation of a trait bound with no runtime dependency at all — both are
exactly as available on a microcontroller target as on a hosted one.

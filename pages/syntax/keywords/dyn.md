---
title: "dyn"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism"]
related_concepts: ["Trait objects & dynamic dispatch (dyn Trait)", "On-stack dynamic dispatch", "Static dispatch & monomorphization"]
related_syntax: [impl, trait]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "On-stack dynamic dispatch", impl]
---

## Explanation

`dyn` marks a type as a **trait object**: `dyn Trait` means "some type
implementing `Trait`, erased and resolved at runtime," rather than a
concrete, statically-known type. Because its size isn't known at compile
time, `dyn Trait` almost always appears behind a pointer —
`&dyn Trait`, `&mut dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>` — never
as a bare, by-value type.

`dyn` is mandatory before a trait used this way (`dyn Shape`, not bare
`Shape`, in a position that means a trait object). This wasn't always
true: before the 2018 edition, `Box<Shape>` and `&Shape` (with no `dyn`)
were legal and meant exactly what `Box<dyn Shape>`/`&dyn Shape` mean now.
That older syntax was ambiguous to read — a bare trait name in type
position looked identical whether it meant "a generic bound" or "a trait
object," and readers (and sometimes the compiler) couldn't always tell
which was meant without more context. The 2018 edition made `dyn`
mandatory specifically to remove that ambiguity: seeing `dyn` in a type
tells you immediately, on sight, "this is a trait object, dispatched at
runtime," with no other reading possible.

Not every trait can be used as `dyn Trait` — the trait has to be
**object-safe** (no generic methods, no method returning `Self` by value,
no associated consts, among other rules). This page doesn't re-derive the
full rule list; see
[Trait objects & dynamic dispatch](../../concepts/traits-polymorphism/trait-objects-dynamic-dispatch.md)
for the mechanism and
[Type erasure](../../concepts/traits-polymorphism/type-erasure-dyn-any.md)
for the broader idea `dyn` is one instance of.

## Usage examples

### Boxing a concrete type as a trait object

```
trait Shape {
    fn area(&self) -> f64;
}
struct Circle;
impl Shape for Circle {
    fn area(&self) -> f64 { 3.14 }
}

let shape: Box<dyn Shape> = Box::new(Circle); // <- `dyn Shape` erases Circle's concrete type
println!("{}", shape.area());
```

### Runtime polymorphism

A notification system that dispatches to whichever channels are
configured at startup needs one collection that can hold genuinely
different types — `dyn` is what makes `Vec<Box<dyn Channel>>` a single,
uniform type despite the elements underneath being different structs.

```
trait Channel {
    fn send(&self, message: &str);
}

struct EmailChannel;
impl Channel for EmailChannel {
    fn send(&self, message: &str) {
        println!("email: {message}");
    }
}

struct SmsChannel;
impl Channel for SmsChannel {
    fn send(&self, message: &str) {
        println!("sms: {message}");
    }
}

fn notify_all(message: &str, channels: &[Box<dyn Channel>]) { // <- `dyn Channel`: one type, many concrete impls
    for channel in channels {
        channel.send(message);
    }
}

let channels: Vec<Box<dyn Channel>> = vec![Box::new(EmailChannel), Box::new(SmsChannel)];
notify_all("order shipped", &channels);
```

The exact mix of channels is only known once
configuration is read at startup, ruling out a fixed enum or generic
function — the canonical case
[Trait objects & dynamic dispatch](../../concepts/traits-polymorphism/trait-objects-dynamic-dispatch.md)
covers in depth, including the vtable mechanism `dyn` triggers.

### Designing a public API

A hot path that dispatches through a trait on every call, but only ever
needs a reference to a value already living on the stack, can use
`&dyn Trait` instead of `Box<dyn Trait>` — no allocation, same runtime
dispatch.

```
trait Logger {
    fn log(&self, message: &str);
}
struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("console: {message}");
    }
}

fn run(logger: &dyn Logger) { // <- `&dyn Logger`: no Box, no heap allocation
    logger.log("started");
}

let console = ConsoleLogger;
run(&console);
```

`dyn` only requires the fat pointer (data + vtable) that
`&`/`&mut` already provide — `Box` is a separate, additional decision
about ownership and heap placement, as
[On-stack dynamic dispatch](../../concepts/design-patterns-idioms/on-stack-dynamic-dispatch.md)
explains in full.

## Explanation (Embedded)

`dyn Trait` works the same under `#![no_std]` as in hosted Rust —
object-safety rules are identical, and a vtable-dispatched call requires
neither an OS nor an allocator by itself. The form that needs nothing
extra is `&dyn Trait` (or `&mut dyn Trait`): a fat pointer — data pointer
plus vtable pointer — to a value already living somewhere (the stack, a
`static`, or borrowed from a caller), with no heap involved at all.
`Box<dyn Trait>` is a different story: `Box` itself is defined in
`alloc`, so boxing a trait object is only available once a crate pulls in
`alloc` and configures a `#[global_allocator]`.

Even where `alloc` is available, many embedded codebases still prefer
generics and monomorphization over `dyn Trait`, for reasons that matter
more on a constrained core than on a server. A generic function is
specialized per concrete type at compile time, so the compiler can inline
across the call and the CPU executes a direct call instead of an
indirect jump through a vtable — a real consideration on a core with a
small or no branch predictor and every cycle budgeted. The tradeoff runs
the other way in code size: monomorphization generates one copy of the
generic code per concrete type actually used, where a single
`dyn`-based function is one copy shared by every implementer — on a chip
with a few tens of kilobytes of flash, that code-size cost is sometimes
the deciding factor back toward `dyn` instead. Neither choice is
universally correct; it's a genuine per-project tradeoff between
dispatch predictability/inlining and flash footprint, not a case where
one option is simply wrong.

## Usage examples (Embedded)

### `&dyn Trait` needs no heap

```
trait Sensor {
    fn read_raw(&self) -> u16;
}

struct Thermistor;
impl Sensor for Thermistor {
    fn read_raw(&self) -> u16 { 512 }
}

fn log_reading(sensor: &dyn Sensor) { // <- `&dyn Sensor`: fat pointer to a borrowed value, no allocation
    let _raw = sensor.read_raw();
}

let thermistor = Thermistor;
log_reading(&thermistor);
```

### `Box<dyn Trait>` once `alloc` is configured

```
extern crate alloc;
use alloc::boxed::Box;

trait Sensor {
    fn read_raw(&self) -> u16;
}

fn boxed_sensor(sensor: impl Sensor + 'static) -> Box<dyn Sensor> {
    Box::new(sensor) // <- `dyn Sensor` here still requires `alloc` + a #[global_allocator], same as hosted Rust
}
```

### Choosing generics over `dyn` for predictable, allocation-free dispatch

```
trait Sensor {
    fn read_raw(&self) -> u16;
}

fn log_reading_generic<S: Sensor>(sensor: &S) { // <- monomorphized per concrete S: no vtable, no dyn at all
    let _raw = sensor.read_raw();
}
```

---
title: "On-stack dynamic dispatch"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Polymorphism"]
related_syntax: [dyn]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "The strategy pattern", "Static dispatch & monomorphization"]
---

## Explanation

A trait object almost always shows up behind `Box` in introductory
examples — `Box<dyn Trait>` — which quietly bundles a heap allocation
into every mention of dynamic dispatch. But the actual mechanism behind
[trait objects](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
is just a fat pointer: a data pointer plus a vtable pointer, and neither
half requires the data itself to live on the heap. `&dyn Trait` and
`&mut dyn Trait` build that same fat pointer around a plain reference to
a value that can live anywhere — most usefully, as an ordinary local on
the stack — which means runtime polymorphism and heap allocation are two
independent decisions, not one, even though beginner examples usually
reach for both at once.

The idiom is to keep concrete values as ordinary stack-local bindings
and reach for `&dyn Trait` only at the single point where one variable
needs to refer to "whichever one of these it turns out to be" —
typically choosing between two or more concrete implementors with an
`if` or `match`, then binding the result through one `&dyn Trait`-typed
variable that outlives the branch. No `Box::new` is involved anywhere:
nothing is heap-allocated, nothing needs a `Drop` to free it, and the
values are cleaned up exactly the way any other stack local would be, at
the end of their scope.

This matters most in two situations: a hot loop where the allocation
itself — not the vtable indirection — would be the actual performance
cost, and any `#![no_std]` environment with no allocator at all, where
`Box<dyn Trait>` simply isn't available but `&dyn Trait` works
unchanged. It's also the right call any time the concrete value's owner
already exists on the stack for the whole span of the call: boxing it
first would add an allocation purely to erase a type that was going to
be dropped at the very same point regardless.

## Basic usage example

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

fn run(logger: &dyn Logger) { // <- takes a reference, never a Box: no heap allocation required
    logger.log("started");
}

let console = ConsoleLogger; // <- an ordinary stack value, not boxed
run(&console);
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

At startup, a CLI reads a `--verbose` flag and must pick one of two
logger implementations for the rest of the run, without allocating just
to hold that choice.

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

struct SilentLogger;
impl Logger for SilentLogger {
    fn log(&self, _message: &str) {}
}

fn configure_logger(verbose: bool) {
    let console = ConsoleLogger; // <- both candidates live on the stack, never boxed
    let silent = SilentLogger;

    let logger: &dyn Logger = if verbose { &console } else { &silent }; // <- one reference binds to either concrete stack value
    logger.log("configuration complete");
}

configure_logger(true);
```

**Why this way:** binding a single `&dyn Logger` to whichever concrete
stack value applies is exactly the technique the
[Rust Design Patterns' on-stack dynamic dispatch idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
names — the branch decides which concrete type is selected, but no
allocation happens anywhere in the process.

### Scenario: Boxing and heap allocation

Reaching for `Box<dyn Trait>` out of habit — when the concrete value
already lives on the stack for the entire call — adds an allocation that
buys nothing over a plain reference.

```
trait Shape {
    fn area(&self) -> f64;
}
struct Circle { radius: f64 }
impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

// AVOID: boxing a value that already lives on the stack for the whole call
fn print_area_boxed(shape: Box<dyn Shape>) {
    println!("{}", shape.area());
}
print_area_boxed(Box::new(Circle { radius: 2.0 })); // <- allocates just to erase the type for one call

// PREFER: a reference erases the type with no allocation at all
fn print_area(shape: &dyn Shape) {
    println!("{}", shape.area());
}
let circle = Circle { radius: 2.0 };
print_area(&circle); // <- no Box, no heap allocation, same runtime dispatch
```

**Why this way:** `Box` is for cases needing genuine ownership transfer
or a lifetime that outlives the current stack frame; when neither
applies, the
[Rust Design Patterns' on-stack dynamic dispatch idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
argues a plain `&dyn Trait` reference gets the same dispatch with none
of the allocation cost.

## Explanation (Embedded)

This idiom is arguably more load-bearing in embedded Rust than anywhere
else in the language, because it resolves a tension that's mostly
theoretical on a hosted target into a genuinely hard constraint on a
microcontroller: static dispatch (generics, monomorphized per concrete
type) costs flash space per instantiation, while `Box<dyn Trait>` costs
an allocator that most `#![no_std]` targets don't have configured at
all. `&dyn Trait` is the one technique that avoids both costs at once —
no heap allocation (it's a fat pointer to a reference, and references can
point at the stack, a `'static`, or anywhere else that isn't the heap),
and no code duplication (one non-generic function processes the trait
object regardless of how many concrete types implement the trait).

The monomorphization side is the one that's easy to underrate: a generic
driver function (`fn run<D: Driver>(driver: &mut D)`) gets compiled
fresh, in full, for every distinct concrete `D` the firmware instantiates
it with. That's usually fine on a desktop where flash isn't a constraint
anyone tracks; on a target with, say, 128 KB of flash shared across an
entire application, instantiating the same generic driver logic for five
sensor variants, three of which are nearly identical, can measurably move
the needle on whether an image fits at all. `&dyn Driver` collapses all
five into one non-generic function using one vtable-dispatched call per
method — trading a small, predictable indirection cost (one pointer
load, one indirect call) for a single compiled copy of the logic instead
of five.

None of this makes generics wrong for embedded code — a driver that's
only ever instantiated once, or where the compiler can inline through the
generic anyway, gets zero benefit from `dyn` and pays the vtable
indirection for nothing. The idiom is choosing `&dyn Trait` specifically
at points with genuine runtime variation (which concrete logger, which
concrete transport) where the alternative really would be code
duplicated across several call sites, not reaching for it as a default.

## Basic usage example (Embedded)

```
trait Transport {
    fn send(&mut self, byte: u8);
}

struct Uart;
impl Transport for Uart {
    fn send(&mut self, byte: u8) {
        let _ = byte; // stands in for a real UART TX register write
    }
}

fn send_all(transport: &mut dyn Transport, data: &[u8]) { // <- &dyn: no heap, works with no allocator configured
    for &byte in data {
        transport.send(byte);
    }
}

let mut uart = Uart; // <- an ordinary stack value, never boxed
send_all(&mut uart, b"boot");
```

## Best practices & deeper information (Embedded)

### Scenario: Runtime polymorphism

A board reads a hardware strap pin at boot to decide whether debug output
should go over UART or a semihosting/RTT channel, and needs one logging
call site that works regardless of which was selected — with no heap
allocator configured on this target at all.

```
trait Logger {
    fn log(&mut self, message: &str);
}

struct UartLogger;
impl Logger for UartLogger {
    fn log(&mut self, message: &str) {
        let _ = message; // stands in for a real UART write
    }
}

struct RttLogger;
impl Logger for RttLogger {
    fn log(&mut self, message: &str) {
        let _ = message; // stands in for a real RTT channel write
    }
}

fn select_logger(debug_strap_high: bool) {
    let mut uart = UartLogger; // <- both candidates live on the stack, never boxed
    let mut rtt = RttLogger;

    let logger: &mut dyn Logger = if debug_strap_high { &mut uart } else { &mut rtt };
    logger.log("boot complete");
}

select_logger(true);
```

**Why this way:** the strap pin's state is only known at runtime, so the
choice between `UartLogger` and `RttLogger` can't be resolved by the
compiler at a single call site the way a generic parameter would be —
`&mut dyn Logger` gets that runtime decision with no allocator, exactly
the technique the [Rust Design Patterns' on-stack dynamic dispatch
idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
describes, applied where `Box<dyn Logger>` would need `alloc` this
target doesn't have configured.

### Scenario: Boxing and heap allocation

On a `#![no_std]` crate with no `alloc` crate linked in at all, `Box<dyn
Trait>` isn't merely discouraged — it doesn't compile, because there is
no global allocator for it to allocate from. On-stack dynamic dispatch is
the direct substitute, not just an optimization over boxing.

```
#![no_std]

trait Sensor {
    fn read(&self) -> i16;
}

struct Thermistor;
impl Sensor for Thermistor {
    fn read(&self) -> i16 { 215 } // stands in for a real ADC read
}

// Box<dyn Sensor> would require the `alloc` crate plus a #[global_allocator] —
// unavailable on this target, so it isn't an option here, not just a discouraged one.

fn print_reading(sensor: &dyn Sensor) { // <- PREFER: &dyn Trait compiles with zero allocator configured
    let _ = sensor.read();
}

let thermistor = Thermistor;
print_reading(&thermistor);
```

**Why this way:** a `#![no_std]` firmware image with no
`#[global_allocator]` has no heap to allocate `Box<dyn Sensor>` from in
the first place, so `&dyn Sensor` isn't a style preference here — it's
the only way to get runtime polymorphism at all; the [Rust Design
Patterns' on-stack dynamic dispatch
idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
frames this as the general no-allocation-cost technique, and embedded
`#![no_std]` code is the setting where it stops being optional.

### Scenario: Writing generic code

A firmware image supports several nearly-identical sensor variants
through one shared trait; instantiating a generic driver function once
per concrete sensor type duplicates the driver's logic in flash for each
one, where a single `&dyn Sensor`-based function compiles once.

```
trait Sensor {
    fn read_raw(&self) -> u16;
}

struct Bme280;
impl Sensor for Bme280 {
    fn read_raw(&self) -> u16 { 4200 }
}

struct Sht31;
impl Sensor for Sht31 {
    fn read_raw(&self) -> u16 { 3900 }
}

// AVOID: a generic function is monomorphized separately for every concrete Sensor it's called with,
// duplicating this logic in flash once per sensor type used in the firmware image.
fn log_reading_generic<S: Sensor>(sensor: &S) {
    let _ = sensor.read_raw();
}

// PREFER: one non-generic function, compiled once, dispatched through a vtable
fn log_reading(sensor: &dyn Sensor) {
    let _ = sensor.read_raw();
}

let bme = Bme280;
let sht = Sht31;
log_reading(&bme); // <- both calls share the same compiled function
log_reading(&sht);
```

**Why this way:** `log_reading_generic::<Bme280>` and
`log_reading_generic::<Sht31>` are two separate copies of identical
logic in the final binary, while `log_reading` compiles once regardless
of how many `Sensor` implementors call into it — trading one indirect
call per invocation for a single copy of the function is a straight
flash-budget win whenever the logic behind the trait is nontrivial and
several concrete types share it, which is exactly the tradeoff [Static
dispatch &
monomorphization](../traits-polymorphism/static-dispatch-monomorphization.md)
covers from the generics side.

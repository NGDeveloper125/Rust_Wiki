---
title: "Marker traits (Send, Sync, Sized, Copy)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism"]
related_syntax: []
see_also: ["Send & Sync", "Copy vs Clone", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

A marker trait has no methods at all — implementing it adds no new
behavior to a type. Its entire purpose is to let the compiler (and other
trait bounds) know a type has some property, purely through the type
system:

- `Copy` marks a type as safe to duplicate with a simple bitwise copy
  (see [Copy vs Clone](../ownership-borrowing/copy-vs-clone.md)).
- `Sized` marks a type whose size is known at compile time — true for
  almost everything, which is why it's an implicit bound on generic
  parameters by default (opted out of with `?Sized`).
- `Send` marks a type as safe to move to another thread.
- `Sync` marks a type as safe to access from multiple threads at once
  through a shared reference (see [Send & Sync](../concurrency-async/send-and-sync.md)).

`Send` and `Sync` are *auto traits*: the compiler implements them
automatically for any type made entirely of parts that already have the
marker — a struct of `Send` fields is automatically `Send` itself, with
no `impl` needed. (`Copy` and `Sized` are different: `Copy` needs an
explicit `#[derive(Copy)]`, and `Sized` is built in.) The auto-trait
mechanism is what lets the compiler enforce thread-safety across an
entire program's type graph without per-type annotations; the property
propagates structurally. A type containing a raw pointer *automatically
loses* `Send`/`Sync` — there is no explicit opt-out on stable Rust
(negative impls are unstable); you suppress the auto-impl by including a
non-`Send` field (e.g. `PhantomData<*const ()>`), and a hand-written
`unsafe impl Send` is how you opt such a type back *in*.

## Basic usage example

```
struct Sensor { reading: i32 } // <- auto-`Send`: every field (i32) is Send, so no impl is needed

fn assert_send<T: Send>() {}
assert_send::<Sensor>();
```

## Best practices & deeper information

### Scenario: Multi-threading

A type shared across threads only compiles where the API requires it if
every field is itself `Send`/`Sync` — the marker traits are what the
compiler checks, not something the code has to assert by hand.

```
struct SensorReading { value: f64, timestamp: u64 } // <- auto-Send/Sync: every field is

fn spawn_worker(reading: SensorReading) {
    std::thread::spawn(move || { // <- requires SensorReading: Send, checked at compile time
        println!("{}: {}", reading.timestamp, reading.value);
    });
}

spawn_worker(SensorReading { value: 21.5, timestamp: 1_700_000_000 });
```

**Why this way:** because `Send`/`Sync` propagate structurally from a
type's fields, most types are thread-safe to move or share automatically
— the
[API Guidelines' C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html)
treats "is `Send`/`Sync` where possible" as a checklist item precisely
because losing it (e.g. by adding an `Rc` or raw pointer field) is easy to
do by accident.

### Scenario: Designing a public API

Manually implementing `Send`/`Sync` is `unsafe` and rare — it's only
needed when a type contains something that isn't automatically
thread-safe (a raw pointer, an FFI handle) and the author can prove, by
hand, that sharing it really is safe.

```
struct FfiHandle(*mut u8); // raw pointer: not auto-Send

// SAFETY: FfiHandle owns its pointer exclusively (no aliases exist), and
// the underlying C object may be used from any one thread at a time, so
// transferring ownership across a thread boundary is sound.
unsafe impl Send for FfiHandle {} // <- manual, unsafe: author is vouching for thread-safety

fn move_to_worker(handle: FfiHandle) {
    std::thread::spawn(move || drop(handle)); // only compiles because of the impl above
}
```

**Why this way:**
[`std::marker::Send`](https://doc.rust-lang.org/std/marker/trait.Send.html)
is an `auto trait` — the compiler derives it automatically for ordinary
types, so a hand-written `unsafe impl` should be rare and deliberate,
reserved for the specific fields (raw pointers, FFI types) that opt a
type out of the automatic derivation.

## Explanation (Embedded)

`Copy`, `Sized`, `Send`, and `Sync` are all defined in `core::marker`, so
none of them depend on `std`, and `Send`/`Sync` remain the canonical
marker-trait example in embedded Rust for the same reason they are on a
hosted target: they answer whether a type is safe to move or share across
a concurrent boundary, purely through the type system, with no runtime
check. The concurrent boundary itself looks different on bare metal —
usually a main loop and one or more `#[interrupt]` handlers rather than
OS threads — which is the full story [Send & Sync's embedded
explanation](../concurrency-async/send-and-sync.md) covers in depth and
isn't repeated here.

A second, genuinely embedded-specific use of the marker-trait idea shows
up inside HAL crates themselves, encoding a hardware capability or pin
mode as a type-level fact rather than a runtime flag. A HAL that models
each GPIO pin's electrical mode as a distinct type (a "type-state" pin)
can also define a marker trait like `PwmCapable` implemented only for the
specific pin/timer-channel combinations whose silicon actually routes to
a PWM channel — a function that configures PWM output can then require
`P: PwmCapable` and reject every other pin at compile time, with zero
runtime cost, instead of accepting any pin and returning an error (or
worse, silently doing nothing) the moment it's asked to drive PWM on a
pin that was never wired to a timer.

## Basic usage example (Embedded)

```
trait PwmCapable {} // marker: no methods, just a type-level fact about this pin

struct Pa0; // this pin's timer channel is routed for PWM
impl PwmCapable for Pa0 {}

struct Pa1; // this pin has no timer channel wired up — no impl

fn configure_pwm<P: PwmCapable>(_pin: P) { /* ... */ }

configure_pwm(Pa0);
// configure_pwm(Pa1); // <- fails to compile: Pa1 doesn't implement PwmCapable
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL exposing PWM configuration should reject non-PWM-capable pins at
compile time rather than at runtime — a zero-method marker trait
implemented only for the pins whose hardware genuinely supports it makes
the constraint part of the function's signature.

```
trait PwmCapable {} // <- marker trait: encodes "this pin's timer channel supports PWM"

struct TimerChannel2Pin; // wired to a timer with PWM output
impl PwmCapable for TimerChannel2Pin {}

struct PlainGpioPin; // no timer routing — deliberately no impl

struct PwmOutput<P: PwmCapable> { // <- only accepts pins that implement the marker
    pin: P,
    duty: u8,
}

impl<P: PwmCapable> PwmOutput<P> {
    fn new(pin: P) -> Self {
        PwmOutput { pin, duty: 0 }
    }
}

let pwm = PwmOutput::new(TimerChannel2Pin); // compiles
// let bad = PwmOutput::new(PlainGpioPin); // <- fails to compile: PlainGpioPin isn't PwmCapable
```

**Why this way:** a wrong pin passed to `PwmOutput::new` becomes a
compile error instead of a runtime misconfiguration that might only
surface as "the LED never dims" — the marker trait costs nothing at
runtime (it has no methods and no fields) while moving a hardware
constraint the type system can express out of the realm of things a test
has to catch.

### Scenario: Multi-threading

A queue type read from the main loop and written from an interrupt
handler needs its element type bounded by `Send`, the same marker trait
[Send & Sync's embedded explanation](../concurrency-async/send-and-sync.md)
covers for sharing state across that boundary — here the constraint
shows up on a generic queue's type parameter rather than on a single
`static`.

```
struct IsrQueue<T: Send, const N: usize> { // <- Send bound: T must be safe to hand across the ISR boundary
    items: [Option<T>; N],
    len: usize,
}

impl<T: Send, const N: usize> IsrQueue<T, N> {
    fn push(&mut self, item: T) -> bool {
        if self.len == N {
            return false;
        }
        self.items[self.len] = Some(item);
        self.len += 1;
        true
    }
}
```

**Why this way:** requiring `T: Send` on the queue itself means any
attempt to store a type that isn't safe to hand across the main-loop/ISR
boundary (an `Rc`, say) fails to compile at the queue's definition site,
rather than only failing later at whichever call site happens to move a
bad value into it — the same structural-propagation guarantee `Send`
provides for OS threads, applied here to a queue shared with an interrupt
handler instead of `std::sync::Mutex`.

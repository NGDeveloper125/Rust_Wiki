---
title: "Dependency injection via traits/generics"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Decoupling"]
related_syntax: []
see_also: ["Traits", "Trait bounds", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

Without a class hierarchy or a dependency-injection framework, Rust
achieves the same decoupling goal — code depending on an abstraction
rather than a concrete implementation — through trait bounds and trait
objects directly: a function can accept `&impl Logger` for a
statically-resolved dependency, or `&dyn Logger` if the concrete type
must vary at runtime.

`run` depends only on "something implementing `Logger`" — a test can pass
in a mock implementation, production code can pass in a real one, and
neither `run` nor `Logger` needs to know which concrete types will ever
implement it. This is the same inversion-of-control idea dependency
injection frameworks in other languages provide via containers and
runtime wiring, but here it's expressed directly in the function
signature and checked entirely at compile time (for `impl Trait`/generic
bounds) or resolved via a simple vtable (for `dyn Trait`) — no separate
framework, reflection, or runtime container needed.

## Basic usage example

```
trait Logger {
    fn log(&self, msg: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, msg: &str) { println!("{msg}"); }
}

fn run(logger: &impl Logger) { // <- depends on the abstraction, not a concrete logger type
    logger.log("started");
}

run(&ConsoleLogger);
```

## Best practices & deeper information

### Scenario: Serving a web endpoint

An axum handler shouldn't hardcode a concrete database or user store —
depending on a trait lets the handler's dependencies be swapped (real
service, mock, different backend) without touching the handler itself.

```
// [dependencies] axum = "0.8", tokio = { version = "1", features = ["full"] }
use axum::extract::State;
use std::sync::Arc;

trait UserStore {
    fn find_name(&self, id: u32) -> Option<String>;
}

struct AppState {
    users: Arc<dyn UserStore + Send + Sync>, // <- handler depends on the trait, not a concrete store
}

async fn get_user(State(state): State<Arc<AppState>>) -> String {
    state.users.find_name(1).unwrap_or_else(|| "unknown".into())
}
```

**Why this way:** wiring the concrete `UserStore` implementation once, at
startup, and injecting only the trait into handlers keeps request-handling
code decoupled from *which* store backs it — the same inversion-of-control
[axum's docs](https://docs.rs/axum/) build shared application state
around via `State<T>` extractors.

### Scenario: Testing

The same trait seam that decouples a handler from its real dependency in
production lets a test swap in an in-memory mock, with no server, network,
or database involved.

```
trait UserStore { // the same seam the handler above depends on
    fn find_name(&self, id: u32) -> Option<String>;
}

struct MockUserStore;
impl UserStore for MockUserStore { // <- test double implementing the same trait as the real store
    fn find_name(&self, id: u32) -> Option<String> {
        if id == 1 { Some("Ada".into()) } else { None }
    }
}

#[test]
fn returns_known_user_name() {
    let store: Box<dyn UserStore> = Box::new(MockUserStore);
    assert_eq!(store.find_name(1), Some("Ada".into()));
}
```

**Why this way:** because handler code only ever calls through
`UserStore`, this test never starts a server or talks to a real database
— the
[Rust Book's mock-object example](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html#a-use-case-for-interior-mutability-mock-objects)
and the wider DI pattern both rely on the trait boundary being the only
thing production and test code have in common.

## Explanation (Embedded)

This pattern isn't just supported in embedded Rust — it's arguably the
central design idea `embedded-hal` exists to enable, and the reason
`trait`-based dependency injection reads as more than an OO-framework
substitute once you're writing firmware. A driver for a sensor, display,
or radio is written generic over an `embedded-hal` trait (`OutputPin`,
`SpiBus`, `I2c`, `DelayNs`, ...) instead of any one chip's concrete HAL
type — see [`trait`'s embedded
explanation](../../syntax/keywords/trait.md) for how that trait boundary
is what lets one driver crate compile unmodified against an STM32 board,
an RP2040 board, or any other target whose HAL implements the traits the
driver needs. Dependency injection is the same shape one level up: the
concrete peripheral implementation is chosen once, at the top of the
application (typically `main`, where the real board's HAL type is known),
and only the trait is threaded down into the driver.

The second payoff is on the host side. Because the driver never names a
concrete chip type, a host-run unit test can construct a fake
implementation of the same trait — a struct that records which pin
operations were called, or returns canned sensor readings — and inject
that instead of real hardware. The driver logic (parsing a sensor's
response bytes, sequencing a display's init commands, retry/backoff
policy) is then exercised by an ordinary `cargo test` on the development
machine, with no debug probe and no board attached, following the same
host-vs-target split [Unit tests' embedded
notes](../../concepts/testing-tooling/unit-tests.md) describe for
hardware-independent logic generally — here the trait seam is *what
makes* the logic hardware-independent in the first place.

## Basic usage example (Embedded)

```
trait OutputPin {
    fn set_high(&mut self);
    fn set_low(&mut self);
}

struct Led<P: OutputPin> { // <- generic over the trait, not any one vendor's pin type
    pin: P,
}

impl<P: OutputPin> Led<P> {
    fn turn_on(&mut self) {
        self.pin.set_high();
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A sensor driver crate should depend on `embedded-hal`'s `I2c` trait, not
on any one vendor's concrete I2C peripheral type, so the same driver
compiles unmodified against boards from different manufacturers.

```
trait I2c {
    fn write_read(&mut self, addr: u8, out: &[u8], in_buf: &mut [u8]);
}

struct TempSensor<I: I2c> { // <- generic over the HAL trait: portable across every chip's I2C impl
    i2c: I,
    addr: u8,
}

impl<I: I2c> TempSensor<I> {
    fn read_celsius(&mut self) -> f32 {
        let mut buf = [0u8; 2];
        self.i2c.write_read(self.addr, &[0x00], &mut buf);
        i16::from_be_bytes(buf) as f32 / 256.0
    }
}
```

**Why this way:** a driver written against `I2c` rather than, say, a
concrete `Stm32I2c1` type runs unchanged on any board whose HAL implements
`I2c` for its own peripheral — the same portability goal [`trait`'s
embedded explanation](../../syntax/keywords/trait.md) covers for
`embedded-hal` as a whole, applied here to one driver crate's own
dependency.

### Scenario: Testing

`TempSensor`'s decoding logic — turning two raw bytes into a Celsius
value — can be verified on the host by injecting a fake `I2c`
implementation that returns a fixed byte pair, with no real sensor or
board attached.

```
trait I2c {
    fn write_read(&mut self, addr: u8, out: &[u8], in_buf: &mut [u8]);
}

struct TempSensor<I: I2c> {
    i2c: I,
    addr: u8,
}

impl<I: I2c> TempSensor<I> {
    fn read_celsius(&mut self) -> f32 {
        let mut buf = [0u8; 2];
        self.i2c.write_read(self.addr, &[0x00], &mut buf);
        i16::from_be_bytes(buf) as f32 / 256.0
    }
}

struct FakeI2c; // test double: no real peripheral involved
impl I2c for FakeI2c {
    fn write_read(&mut self, _addr: u8, _out: &[u8], in_buf: &mut [u8]) {
        in_buf.copy_from_slice(&[25, 0]); // canned reading: 25.0 C
    }
}

#[test]
fn decodes_a_fixed_reading_as_celsius() {
    let mut sensor = TempSensor { i2c: FakeI2c, addr: 0x48 };
    assert_eq!(sensor.read_celsius(), 25.0);
}
```

**Why this way:** because `TempSensor` only ever calls through `I2c`,
this test runs as an ordinary host-side `cargo test` with no target
hardware — the same fast, deterministic split [Unit tests' embedded
notes](../../concepts/testing-tooling/unit-tests.md) describe for
hardware-independent logic, made possible here specifically because the
driver was written against the trait rather than a concrete peripheral
type.

---
title: "The typestate pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns", "Design Patterns & Idioms", "State Machines", "Designing Robust Data Models"]
related_syntax: []
see_also: ["The builder pattern", "Enums (algebraic data types)", "Structs"]
---

## Explanation

The typestate pattern encodes a value's state as part of its *type*, so
a method that's illegal in a given state simply doesn't exist for that
type — the compiler rejects an illegal call at compile time, instead of
the program discovering the mistake at runtime through a panic or an
`if` check against some internal `state` field. Each state gets its own
type, often zero-sized; a transition method takes `self` by value and
returns the type for the next state, both performing the transition and
consuming the old state so it can't be reused out of order.

Concretely: a network connection can be represented as a `Disconnected`
value and a `Connected` value — two distinct types, rather than one
`Connection` struct with a `state: ConnectionState` field. `fn
connect(self: Disconnected) -> Connected` is the only way to obtain a
`Connected` value, and there is no `send` method on `Disconnected` at
all, so calling `send` before connecting isn't a bug caught by a runtime
assertion — it's a type error the compiler reports immediately. This
takes the "make invalid states unrepresentable" idea one step further
than an [enum](../types-data-modeling/enums-algebraic-data-types.md)
alone: instead of only making invalid *data* impossible to construct, it
makes invalid *sequences of operations* impossible to write.

The tradeoff is that typestate only works when the valid states and
legal transitions are fixed at compile time and don't depend on data the
program only learns at runtime. If which transition is legal depends on
something read from a config file or a database, the type of a value
can't vary with that at runtime — an enum-tagged state checked with
`match` is the right tool there instead, trading away the compile-time
guarantee for runtime-decided flexibility.

Typestate is frequently combined with [the builder pattern](the-builder-pattern.md):
a builder's own type changes as required fields are supplied, so
`.build()` only compiles once every mandatory field has actually been
set — the same "illegal states aren't just checked, they don't exist"
idea applied to object construction specifically.

## Basic usage example

```
struct Disconnected;
struct Connected { session_id: u32 }

impl Disconnected {
    fn connect(self) -> Connected { // <- consumes Disconnected: the only way to obtain a Connected value
        Connected { session_id: 1 }
    }
}

impl Connected {
    fn send(&self, message: &str) {
        println!("session {}: {message}", self.session_id);
    }

    fn disconnect(self) -> Disconnected { // <- consumes Connected: no session-less send() is reachable afterward
        Disconnected
    }
}

let conn = Disconnected;
let conn = conn.connect();
conn.send("hello");
// conn.disconnect().send("oops"); // would fail to compile: Disconnected has no `send` method
```

## Best practices & deeper information

### Scenario: Designing a public API

An order shouldn't be shippable before it's paid; encoding "paid" as a
distinct type rather than a boolean field means there is no `ship`
method to accidentally call on an order that hasn't been paid for.

```
struct UnpaidOrder { id: u32, total_cents: u32 }
struct PaidOrder { id: u32 }

impl UnpaidOrder {
    fn pay(self, amount_cents: u32) -> Result<PaidOrder, UnpaidOrder> {
        if amount_cents >= self.total_cents {
            Ok(PaidOrder { id: self.id }) // <- transition consumes the unpaid order; only a PaidOrder can ship
        } else {
            Err(self) // <- insufficient payment: hand the same order back, still unpaid
        }
    }
}

impl PaidOrder {
    fn ship(&self) {
        println!("shipping order {}", self.id);
    }
}

let order = UnpaidOrder { id: 42, total_cents: 500 };
if let Ok(paid) = order.pay(500) {
    paid.ship(); // <- only reachable once `pay` has succeeded
}
```

**Why this way:** there is no `ship` method on `UnpaidOrder` at all, so
"ship before paying" isn't a bug that can even be written, let alone one
that needs a runtime guard — exactly the guarantee the
[Rust Design Patterns' typestate entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/typestate.html)
describes.

### Scenario: Validating input

A signup form's fields need validating exactly once before any
downstream code can trust them; typestate expresses "validated" as a
distinct type instead of a boolean flag sitting next to the raw data.

```
struct RawSignup { email: String }
struct ValidSignup { email: String }

impl RawSignup {
    fn validate(self) -> Result<ValidSignup, &'static str> {
        if self.email.contains('@') {
            Ok(ValidSignup { email: self.email }) // <- downstream code only ever sees this type, never the raw one
        } else {
            Err("invalid email")
        }
    }
}

fn create_account(signup: ValidSignup) { // <- signature alone guarantees validation already happened
    println!("account created for {}", signup.email);
}

let raw = RawSignup { email: "user@example.com".to_string() };
if let Ok(valid) = raw.validate() {
    create_account(valid);
}
```

**Why this way:** because `create_account` only accepts a `ValidSignup`,
every caller gets the validation guarantee for free without re-checking
— the parse-don't-validate argument
[Effective Rust](https://effective-rust.com/) makes for pushing checks to
the type system instead of scattering `if` checks at every use site.

### Scenario: Branching on data (pattern matching)

A resumable background job's next legal step depends on a value loaded
from a database at runtime, so the type of the job value can't encode
it — an enum with a `match` is the right tool here, not typestate.

```
enum JobState {
    Pending,
    Running { progress_percent: u8 },
    Done,
}

fn resume(state: JobState) { // <- state is only known once the record is loaded at runtime
    match state { // <- runtime branching plays typestate's role, since the type can't vary with loaded data
        JobState::Pending => println!("starting job"),
        JobState::Running { progress_percent } => println!("resuming at {progress_percent}%"),
        JobState::Done => println!("nothing to do"),
    }
}

resume(JobState::Running { progress_percent: 40 });
```

**Why this way:** typestate's compile-time guarantee only applies to
transitions the compiler can see at each call site; once the legal next
state depends on data read at runtime, matching an exhaustive
[enum](../types-data-modeling/enums-algebraic-data-types.md) is the
honest tool, not a workaround.

## Explanation (Embedded)

Typestate is one of the strongest, most genuinely idiomatic fits between
a design pattern and the embedded domain in this entire catalog — not a
stretch application of a hosted-Rust idea, but a pattern the
`embedded-hal` ecosystem actively builds around. The canonical example is
a GPIO pin: a microcontroller pin can be configured as a floating input,
a pulled-up input, a push-pull output, an open-drain output, or an analog
input, and writing to a pin that's currently configured as an input, or
reading an ADC-style analog value from a pin configured as a digital
output, isn't a logic error a test suite might happen to catch — it's a
category of bug that shows up as a runtime hardware fault: a bus fault,
a pin that silently never toggles, a floating input read as noise. HAL
crates like `stm32f4xx-hal`, `rp2040-hal`, and others encode the pin's
mode as a type parameter — `Pin<'A', 5, Input>`, `Pin<'A', 5, Output>` —
so that the *type itself* records which mode the pin is in, and mode-
specific methods (`set_high`/`set_low` on an output, `is_high` on an
input) simply don't exist on the wrong type. A pin-mode-change method
like `into_output()` takes the input-mode pin by value and returns a
new, output-typed pin, both performing the underlying register write
that reconfigures the mode *and* consuming the old typed handle so it
can't be used in its former mode afterward. Because the marker types
(`Input`, `Output`, `Analog`, and similar) are zero-sized, none of this
costs anything at runtime beyond the register write the transition
method actually performs — the type-level bookkeeping vanishes entirely
after compilation, leaving only the actual hardware configuration
instruction. The same idea extends past GPIO to peripheral
initialization generally: a timer, ADC, or DMA channel's "configured but
not yet started" versus "running" states are a natural typestate split,
turning "started a peripheral twice" or "read from an unconfigured ADC"
into compile errors instead of undefined hardware behavior.

## Basic usage example (Embedded)

```
struct Input;
struct Output;

struct Pin<MODE> {
    number: u8,
    _mode: core::marker::PhantomData<MODE>,
}

impl Pin<Input> {
    fn into_output(self) -> Pin<Output> { // <- consumes the input-mode pin: the only way to obtain an output-mode one
        Pin { number: self.number, _mode: core::marker::PhantomData }
    }
}

impl Pin<Output> {
    fn set_high(&self) {
        // volatile write to the pin's output-data register
    }
}

let pin: Pin<Input> = Pin { number: 5, _mode: core::marker::PhantomData };
let pin = pin.into_output();
pin.set_high();
// pin.is_high(); // would fail to compile: Pin<Output> has no `is_high` method
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL crate's GPIO pin type should make it impossible to call
`set_high` on a pin still configured as an input — encoding the mode as
a type parameter means there is no `set_high` method to accidentally
reach for on an `Input`-typed pin at all.

```
struct Input;
struct Output;

struct Pin<MODE> {
    number: u8,
    _mode: core::marker::PhantomData<MODE>,
}

impl Pin<Input> {
    fn into_output(self) -> Pin<Output> { // <- reconfigures the pin's mode register and changes its type together
        Pin { number: self.number, _mode: core::marker::PhantomData }
    }
}

impl Pin<Output> {
    fn set_high(&self) {
        // volatile write to the pin's output-data register
    }
    fn into_input(self) -> Pin<Input> {
        Pin { number: self.number, _mode: core::marker::PhantomData }
    }
}

let led: Pin<Input> = Pin { number: 13, _mode: core::marker::PhantomData };
let led = led.into_output();
led.set_high(); // <- only reachable once into_output() has actually run
```

**Why this way:** there is no `set_high` method on `Pin<Input>` at all,
so "write to a pin still wired as an input" isn't a bug that can even be
written, let alone one caught only by testing on real hardware — this is
the exact guarantee the
[Rust Design Patterns' typestate entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/typestate.html)
describes, and it's why typestate is the default way `embedded-hal`-
ecosystem crates model pin modes today.

### Scenario: Validating input

An ADC needs a one-time calibration step before any reading from it can
be trusted; typestate expresses "calibrated" as a distinct type instead
of a boolean flag next to the raw peripheral handle, so a reading can't
be taken from an uncalibrated ADC by mistake.

```
struct Uncalibrated;
struct Calibrated { offset: i16 }

struct Adc<STATE> {
    _state: STATE,
}

impl Adc<Uncalibrated> {
    fn calibrate(self) -> Adc<Calibrated> {
        let offset = 0; // stand-in for a real calibration routine
        Adc { _state: Calibrated { offset } } // <- downstream code only ever sees a Calibrated Adc
    }
}

impl Adc<Calibrated> {
    fn read(&self) -> i16 { // <- signature alone guarantees calibration already happened
        1024 - self._state.offset
    }
}

let adc = Adc { _state: Uncalibrated };
let adc = adc.calibrate();
let _reading = adc.read();
```

**Why this way:** because `read` only exists on `Adc<Calibrated>`, every
caller gets the calibration guarantee for free without re-checking a
flag at every call site — the same parse-don't-validate argument
[Effective Rust](https://effective-rust.com/) makes for pushing
correctness checks into the type system instead of scattering `if`
checks through firmware code that reads the ADC.

### Scenario: Branching on data (pattern matching)

A pin's alternate-function mux (UART TX vs. SPI MOSI vs. plain GPIO) is
selected by a byte loaded from external configuration at boot, so the
choice genuinely isn't known at compile time — an enum with a `match` is
the honest tool here, not typestate.

```
enum PinFunction {
    Gpio,
    UartTx,
    SpiMosi,
}

fn configure_mux(function: PinFunction) { // <- function is only known once configuration is loaded at boot
    match function { // <- runtime branching plays typestate's role, since the type can't vary with loaded config
        PinFunction::Gpio => { /* set mux bits for GPIO */ }
        PinFunction::UartTx => { /* set mux bits for UART TX */ }
        PinFunction::SpiMosi => { /* set mux bits for SPI MOSI */ }
    }
}

configure_mux(PinFunction::UartTx);
```

**Why this way:** typestate's compile-time guarantee only helps when the
compiler can see the transition at each call site; once the legal pin
function depends on a byte read from configuration at boot, matching an
exhaustive [enum](../types-data-modeling/enums-algebraic-data-types.md)
is the honest tool — reaching for typestate here would just mean writing
a runtime `if`/`match` to pick which typestate method to call, with no
compile-time benefit left to show for it.

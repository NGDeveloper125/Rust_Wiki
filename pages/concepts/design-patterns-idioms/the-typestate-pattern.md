---
title: "The typestate pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "State Machines", "Designing Robust Data Models"]
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

## Embedded Rust Notes

**Full support.** Typestate costs nothing at runtime — the marker types
for each state are typically zero-sized and optimize away entirely, and
the checking happens purely at compile time. The pattern is especially
popular in embedded HAL crates, where a GPIO pin's mode (`Input`,
`Output`, `Analog`) is encoded as a type parameter so configuring a pin
incorrectly for how it's used is a compile error rather than a runtime
hardware fault.

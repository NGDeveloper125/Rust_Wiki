---
title: "Immutability by default"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Functional Programming", "Coming from Python / JavaScript"]
related_syntax: [let, mut]
see_also: ["Ownership", "Mutable borrowing"]
---

## Explanation

Every binding introduced with `let` is immutable unless explicitly marked
`mut`. This is the opposite default from most mainstream languages (Java,
Python, JavaScript, C), where variables are mutable unless specially
declared `const`/`final`.

The practical effect is that immutability becomes the norm you reach for,
and mutability becomes something you opt into deliberately at each
binding site — a small, local signal to a reader that *this* particular
variable is expected to change, which makes the ones that don't stand out
as safe to reason about without tracking their value over time.

This default also interacts with the borrow checker directly: a shared
reference (`&T`) cannot be used to mutate through it (unless the type
opts into interior mutability — `Cell`, `RefCell`, `Mutex`, atomics — see
[Interior mutability](interior-mutability.md)), precisely
because immutability-by-default is the baseline the whole borrowing model
is built on top of — mutability is the special case that needs an
explicit `&mut` to unlock, not the other way around. This is a large part
of why data races are ruled out at compile time: outside of those
interior-mutability types, you cannot have two
simultaneous mutable accesses to the same data without the compiler
seeing an explicit `&mut` for it.

## Basic usage example

```
let x = 5;
// x = 6; // would fail to compile: x is immutable by default

let mut y = 5;
y = 6; // <- `mut` explicitly opts this binding into reassignment
println!("{x} {y}");
```

## Best practices & deeper information

### Scenario: Creating a new object

Constructing a fully-formed `Order` in one expression, rather than
creating a default and mutating fields into place afterward, means the
binding never needs `mut` at all.

```
struct Order {
    id: u64,
    total_cents: u64,
    status: &'static str,
}

// AVOID: default-then-mutate needs `mut` and leaves a window where `order` is incompletely formed
// let mut order = Order { id: 0, total_cents: 0, status: "" };
// order.id = 42;
// order.total_cents = 1999;
// order.status = "pending";

// PREFER: build the finished value in one immutable binding
let order = Order { id: 42, total_cents: 1999, status: "pending" }; // <- no `mut` needed: fully formed up front
```

**Why this way:** constructing a value complete at its point of creation
means there's no intermediate state where the struct exists but is only
half-initialized — the
[Rust Book](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
frames immutable-by-default as nudging code toward exactly this shape,
reserving `mut` for values that genuinely change over their lifetime.

### Scenario: Designing a public API

A public config type that hands back a new, independent value from an
"update" — instead of exposing mutable setters — means no caller has to
worry about a value changing out from under them after they've stored it.

```
#[derive(Clone)]
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u32,
}

impl RetryPolicy {
    fn with_max_attempts(&self, max_attempts: u32) -> Self { // <- PREFER: returns a new, independent value
        Self { max_attempts, ..self.clone() }
    }
}

// AVOID: a public `set_max_attempts(&mut self, ...)` lets any holder mutate a policy others may rely on

let default_policy = RetryPolicy { max_attempts: 3, backoff_ms: 100 };
let aggressive = default_policy.with_max_attempts(10); // default_policy is untouched
```

**Why this way:** an API built around producing new immutable values
instead of mutating shared ones means no caller has to track whether a
`RetryPolicy` they're holding might change later — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)
favor predictable, non-surprising behavior, and an immutable-by-default
value type is the simplest way to guarantee it.

## Explanation (Embedded)

The immutable-by-default rule is enforced identically on every target —
no `std`/allocator dependency, nothing changes under `#![no_std]`. Where
it earns its keep especially in embedded code is around hardware
configuration values: a peripheral config (baud rate, clock prescaler,
ADC gain, GPIO mode) is typically built completely and then handed once
to a HAL `init`/`new` call that consumes it and programs the actual
hardware registers from it. Once that config has been "committed" to
silicon, there's no code path where mutating the original binding would
have any further effect — the hardware doesn't watch a config struct for
changes, it was read once at init time. A binding that's immutable by
default communicates that directly: if the value can never be reassigned
after construction, nobody reading the surrounding code needs to search
for a place it might quietly drift out of sync with the hardware it
configured. This heads off a real, common bug class in embedded C, where
a config struct is a mutable global that other code can innocently poke
after init, silently desyncing software's idea of the configuration from
what's actually latched into the peripheral's registers — Rust's default
makes that class of accidental "config drift" a compile error the moment
a stray assignment to a non-`mut` binding is attempted.

## Basic usage example (Embedded)

```
struct UartConfig { baud_rate: u32, stop_bits: u8 }

let config = UartConfig { baud_rate: 115_200, stop_bits: 1 }; // <- no `mut`: never touched again
// config.baud_rate = 9600; // would fail to compile

let serial = Serial::init(config); // consumes the fully-formed config, programs the hardware once
```

## Best practices & deeper information (Embedded)

### Scenario: Creating a new object

Build the UART config completely before handing it to `init`, rather than
starting from a default and mutating fields into place.

```
struct UartConfig { baud_rate: u32, stop_bits: u8, parity: bool }

// AVOID: default-then-mutate needs `mut` and leaves a window where `config`
// doesn't yet match what will actually be programmed into the peripheral
// let mut config = UartConfig { baud_rate: 9600, stop_bits: 1, parity: false };
// config.baud_rate = 115_200;
// config.parity = true;

// PREFER: build the finished config in one immutable binding
let config = UartConfig { baud_rate: 115_200, stop_bits: 1, parity: true }; // <- no `mut` needed

let serial = Serial::init(config); // <- consumes the config exactly once; nothing can drift after this
```

**Why this way:** a config value that's complete the moment it's created
has no window where its fields don't yet match what's about to be
programmed into the peripheral — the same reasoning the
[Rust Book](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
gives for immutable-by-default applies with extra weight here, since the
"half-built" state it avoids would otherwise be a config that looks valid
in software but doesn't yet match the hardware it's about to configure.

### Scenario: Designing a public API

The clock-configuration "freeze" pattern common to Cortex-M HAL crates: a
builder is assembled, then consumed by a call that commits it to hardware
and hands back an immutable value with no way to mutate it afterward.

```
struct ClockConfig { sysclk_hz: u32, pclk1_hz: u32 }

impl ClockConfig {
    fn sysclk(mut self, hz: u32) -> Self { self.sysclk_hz = hz; self } // <- returns a new value, not &mut self
    fn freeze(self) -> Clocks { // <- consumes self: commits the config to the actual clock hardware
        Clocks { sysclk_hz: self.sysclk_hz, pclk1_hz: self.pclk1_hz }
    }
}

struct Clocks { sysclk_hz: u32, pclk1_hz: u32 } // <- no setters: a frozen Clocks can't drift from the hardware it describes

let clocks = ClockConfig { sysclk_hz: 8_000_000, pclk1_hz: 8_000_000 }
    .sysclk(48_000_000)
    .freeze(); // <- committed; reconfiguring means building a new ClockConfig, not mutating this one
```

**Why this way:** exposing `Clocks` as an immutable value with no setters
means every later reader can trust `clocks.sysclk_hz` actually matches
what the hardware is running at, instead of having to double-check
whether some other code path mutated it after `freeze()` ran — the same
"predictable, non-surprising API" reasoning the
[API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)
give for returning new values instead of exposing mutable setters, applied
here to a value a physical clock peripheral has already been configured
from.

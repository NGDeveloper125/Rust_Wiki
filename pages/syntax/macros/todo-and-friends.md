---
title: "todo! / unimplemented! / unreachable!"
kind: macro
embedded_support: full
groups: ["Errors & Assertions", "Macros & Metaprogramming"]
related_concepts: ["Panic & unwinding", "Exhaustiveness checking"]
related_syntax: ["panic!", "match"]
see_also: ["panic!"]
---

## Explanation

All three macros panic unconditionally and accept the same optional
format-string message as [`panic!`](panic-macro.md) — functionally,
`todo!()`, `unimplemented!()`, and `unreachable!()` are just `panic!` with
a fixed default message. What differs is entirely what each one
*communicates* to a future reader (including a future you), which is the
only reason to pick one over another, or over a plain `panic!`.

`todo!()` says "this is temporarily missing — I intend to write it." It's
a stub for code that doesn't exist yet: a trait method being filled in
incrementally, a match arm for a case not yet handled. Its presence in
code under active development is expected and unremarkable; its presence
in a shipped build usually isn't.

`unimplemented!()` says "this is permanently missing — by design, not by
oversight." It marks a gap that isn't going away with more work: one
method of a trait that a specific implementation deliberately doesn't
support (a read-only storage backend whose `delete` method is
`unimplemented!()` because deletion was never meant to be supported
there), documented as a real limitation rather than a work-in-progress
marker.

`unreachable!()` says something different again: "the program's own
logic guarantees control flow cannot arrive here." It isn't a gap at all
— it's an assertion that a branch is dead given everything already
established earlier in the function (an exhaustive `match` where one arm
is kept only as a defensive default after every real case has already
been handled, or code reached only after a prior check has already ruled
a value out). Reaching it means an invariant the code relies on was
actually false — a genuine bug, not missing work.

## Usage examples

### Marking a permanently unsupported method

```
trait Storage {
    fn read(&self, key: &str) -> Option<String>;
    fn delete(&self, key: &str);
}

struct ReadOnlyStorage;

impl Storage for ReadOnlyStorage {
    fn read(&self, key: &str) -> Option<String> {
        let _ = key;
        None
    }

    fn delete(&self, _key: &str) {
        unimplemented!("ReadOnlyStorage does not support deletion by design") // <- permanent, not a stub
    }
}
```

### Designing a public API

A `Shape` trait's area method is implemented for every variant except one
still being built out — `todo!()` marks the honest gap during
development, distinct from `unimplemented!()`, which would claim the gap
is permanent.

```
trait Shape {
    fn area(&self) -> f64;
}

struct Circle { radius: f64 }
struct Polygon { vertices: Vec<(f64, f64)> } // shoelace-formula area not yet ported over

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Shape for Polygon {
    fn area(&self) -> f64 {
        todo!("port the shoelace-formula area calculation from the old prototype") // <- work-in-progress, not permanent
    }
}
```

[Effective Rust](https://effective-rust.com/) treats
`todo!()` during active development as a signal to reviewers and
teammates that the gap is expected to close, which `unimplemented!()`
would misleadingly suggest is a deliberate, final design choice instead.

### Branching on data (pattern matching)

An exhaustive `match` over an order's state machine has already handled
every real transition; the final arm exists only as a safety net the type
system can't express, backed by `unreachable!()`.

```
enum OrderState {
    Placed,
    Shipped,
    Delivered,
}

fn next_state(state: OrderState) -> OrderState {
    match state {
        OrderState::Placed => OrderState::Shipped,
        OrderState::Shipped => OrderState::Delivered,
        OrderState::Delivered => unreachable!("a delivered order has no next state and should never be advanced again"),
        // <- asserts this call site never advances an already-delivered order
    }
}
```

`unreachable!()` documents an invariant enforced
elsewhere in the program (callers are expected to check for
`OrderState::Delivered` before calling `next_state`), turning a silent
logic bug into a loud, immediate panic instead of quietly continuing on
bad state — the
[Reference's exhaustiveness rules](https://doc.rust-lang.org/reference/expressions/match-expr.html)
require every arm to be present, but only the code's own logic can
guarantee one of them never actually runs.

## Explanation (Embedded)

All three macros are thin wrappers over `core::panic!`, so they carry no
`std` dependency and their calling convention is identical under
`#![no_std]` — what fires when any of them is reached is exactly the
[`#[panic_handler]`](../attributes/panic-handler-attribute.md) function
the binary supplies, the same as a direct `panic!` call; see
[`panic!`](panic-macro.md) for why bare metal needs one and what crates
like `panic-halt`/`panic-probe` do once it runs. The three macros'
distinct meanings — "not written yet," "permanently unsupported,"
"provably dead code" — carry over to embedded code unchanged, and if
anything matter more there: a `todo!()` left in a trait method of a
peripheral driver, or an `unreachable!()` guarding a state machine driven
by interrupt handlers, panics identically to a hosted build, just by way
of the embedded panic handler instead of a hosted unwind.

## Usage examples (Embedded)

### Marking an unimplemented HAL trait method

```
trait Pwm {
    fn set_duty(&mut self, percent: u8);
    fn set_dead_time(&mut self, ns: u32);
}

struct BasicTimerPwm;

impl Pwm for BasicTimerPwm {
    fn set_duty(&mut self, percent: u8) {
        let _ = percent;
        // ... write to the timer's compare register
    }

    fn set_dead_time(&mut self, _ns: u32) {
        unimplemented!("this timer instance has no complementary outputs, so dead-time insertion is not supported")
        // <- permanent hardware limitation, not a stub
    }
}
```

### Asserting an interrupt-driven state machine invariant

An interrupt handler advances a link-state machine; the fallback arm is
reachable only if the state machine's own invariants were violated
elsewhere, so it panics via `unreachable!()` rather than continuing
silently in a program state that shouldn't exist.

```
enum LinkState {
    Down,
    Negotiating,
    Up,
}

fn on_link_interrupt(state: LinkState) -> LinkState {
    match state {
        LinkState::Down => LinkState::Negotiating,
        LinkState::Negotiating => LinkState::Up,
        LinkState::Up => unreachable!("link-up interrupt fired again while already up"),
        // <- see panic! for what actually runs here on bare metal: the #[panic_handler]
    }
}
```

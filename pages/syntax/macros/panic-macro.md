---
title: "panic!"
kind: macro
embedded_support: full
groups: ["Errors & Assertions", "Macros & Metaprogramming"]
related_concepts: ["Panic & unwinding"]
related_syntax: ["!", "#[panic_handler]"]
see_also: ["Panic & unwinding", "#[panic_handler]"]
---

## Explanation

`panic!` triggers an unrecoverable error in the current thread. With no
arguments, `panic!()` panics with a generic default message; with a
string literal, `panic!("message")` panics with that exact text; and with
a format string and arguments, `panic!("bad value: {value}")`, it panics
with a message built through the exact same formatting grammar as
`format!`/`println!` (see [`format!`](format-macro.md) for that syntax)
— the message is only ever formatted if the panic actually fires, so
`panic!("failed: {}", expensive_debug_string())` never pays for the
formatting on the non-panicking path.

Grammatically, `panic!(...)` is an expression of the
[never type `!`](../operators/exclamation-mark.md), which is why it
type-checks in any position an ordinary value is expected — a `match`
arm, the tail of an `if`/`else`, the body of a function declared to
return some concrete `T`. The compiler doesn't need a real value there,
because control flow never reaches past the `panic!` call.

This page covers the macro's own calling convention; what actually
happens once it fires — unwinding versus aborting, `Drop` running on the
way out, its interaction with threads and `catch_unwind`, and when to
reach for `panic!` over returning a `Result` — is the concept-level story
covered in
[Panic & unwinding](../../concepts/error-handling/panic-and-unwinding.md).

## Usage examples

### Panicking with a formatted message

```
fn set_volume(level: u8) {
    if level > 100 {
        panic!("volume {level} exceeds the maximum of 100"); // <- formatted message, same grammar as format!/println!
    }
    // ...
}
```

### Handling and propagating errors

A payment gateway's internal charge function documents and enforces a
precondition — the amount must already have been validated as positive by
the caller — and panics with a formatted message naming exactly which
value violated it, rather than silently clamping or returning a sentinel.

```
struct PaymentGateway;

impl PaymentGateway {
    /// Panics if `amount_cents` is not positive — callers are expected to
    /// validate user input before reaching this point.
    fn charge(&self, amount_cents: i64) {
        if amount_cents <= 0 {
            panic!("charge amount must be positive, got {amount_cents} cents"); // <- names the exact bad value in the message
        }
        // ... submit the charge
    }
}

let gateway = PaymentGateway;
gateway.charge(1999);
```

The message is written for the developer who trips this
precondition during development, not an end user, so it names the
concrete value rather than a generic "invalid amount" — the API
Guidelines'
[C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#function-docs-include-error-panic-and-safety-considerations-c-failure)
item is why this is documented as a precondition instead of handled some
other way.

### Testing

A `should_panic` test locks in that the exact formatted message from a
`panic!` call names the field that failed validation, so a future
refactor can't silently change what's reported without the test noticing.

```
struct DeviceConfig {
    channel: u8,
}

impl DeviceConfig {
    fn new(channel: u8) -> Self {
        if channel == 0 {
            panic!("channel must be nonzero, got {channel}"); // <- exact wording asserted against below
        }
        DeviceConfig { channel }
    }
}

#[test]
#[should_panic(expected = "channel must be nonzero")]
fn rejects_zero_channel() {
    DeviceConfig::new(0);
}
```

The
[Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)
recommends `should_panic(expected = ...)` over a bare `should_panic`
specifically so pinning down the `panic!` message's wording, not merely
that a panic occurs, catches a regression where the check still fires but
for a different, wrong reason.

## Embedded Rust Notes

**Full support.** `panic!` is `core::panic!` — invoking it has no `std`
dependency. What changes under `#![no_std]` is what happens afterward:
there's no default unwind-and-print-a-backtrace behavior, so every
`#![no_std]` binary must supply exactly one function marked
[`#[panic_handler]`](../attributes/panic-handler-attribute.md), and most
embedded targets build with `panic = "abort"` rather than unwinding at
all — see
[Panic & unwinding](../../concepts/error-handling/panic-and-unwinding.md)
for that distinction.

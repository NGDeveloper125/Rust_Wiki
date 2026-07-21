---
title: "#[should_panic]"
kind: attribute
embedded_support: partial
groups: ["Testing", "Testing & Tooling"]
related_concepts: ["Unit tests"]
related_syntax: ["#[test]", "#[ignore]"]
see_also: ["#[test]"]
---

## Explanation

`#[should_panic]` is placed alongside `#[test]` on a test function and
inverts what counts as passing: the test **fails if the function returns
normally**, and **passes only if it panics**. This is the tool for
asserting a negative contract — "this input is invalid, and calling this
function with it must panic" — where an ordinary `assert!`-based test
can't express the expectation, since there's no return value to assert
against once the function panics before returning anything at all.

The optional `expected = "substring"` form adds a second condition: the
panic must additionally carry a message **containing** that substring, or
the test still fails even though a panic did occur. This catches a subtle
false-positive a bare `#[should_panic]` would miss — the function
panicking for the *wrong* reason (say, an unrelated bug causing an
out-of-bounds index) still satisfies a bare `#[should_panic]`, since any
panic at all passes it, but would not match a specific expected message
naming the intended validation failure. `expected` doesn't require an
exact match, only that the panic message contains the given text
somewhere, which keeps the test from breaking over an unrelated wording
tweak elsewhere in the message.

## Usage examples

### Asserting a panic with an expected message

```
struct OrderId(u32);

impl OrderId {
    fn new(value: u32) -> Self {
        if value == 0 {
            panic!("order id must be non-zero");
        }
        OrderId(value)
    }
}

#[test]
#[should_panic(expected = "order id must be non-zero")] // <- passes only if this exact panic occurs
fn rejects_zero_order_id() {
    OrderId::new(0);
}
```

### Testing

A constructor that panics on invalid input — rather than returning
`Result` — needs its panic behavior tested just as much as a
`Result`-returning one needs its `Err` branch tested; `#[should_panic]`
with `expected` confirms both that it panics *and* that it panics for the
validation reason the test intends, not some unrelated bug.

```
struct PositiveQuantity(u32);

impl PositiveQuantity {
    fn new(value: u32) -> Self {
        assert!(value > 0, "quantity must be positive, got {value}");
        PositiveQuantity(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_positive_quantity() {
        let q = PositiveQuantity::new(3);
        assert_eq!(q.0, 3);
    }

    #[test]
    #[should_panic(expected = "quantity must be positive")] // <- narrows the pass condition to THIS panic
    fn rejects_zero_quantity() {
        PositiveQuantity::new(0);
    }
}
```

Without `expected`, `rejects_zero_quantity` would also
"pass" if `PositiveQuantity::new` panicked for a completely unrelated
reason — an integer overflow elsewhere in the function, say — masking a
real bug behind a green test; the
[Rust Book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)
recommends the `expected` argument specifically so a `#[should_panic]`
test verifies the *intended* failure, not merely that some panic occurred.

## Explanation (Embedded)

`#[should_panic]` is a modifier on `#[test]`, which needs the host-run
test harness described in [`#[test]`](test-attribute.md)'s Embedded Rust
Notes — catching a panic and reporting pass/fail assumes an
unwinding-capable, hosted process, which a bare `#![no_std]` target build
doesn't have by default (many embedded configurations use
`panic = "abort"` and a panic handler that resets or halts the chip, not
one that unwinds back into a test harness). Hardware-independent logic
split out for host testing — a parser rejecting a malformed frame, say —
can still use `#[should_panic]` completely normally when it's compiled
and run on the host toolchain, same as any other crate. There's no
on-target equivalent: an on-target framework like `defmt-test` reports
each test's pass/fail over RTT but doesn't provide a
"this function is expected to panic" mechanism of its own — asserting a
panic on real hardware would mean recovering from an abort/reset mid
test-run, which these harnesses aren't built around.

## Usage examples (Embedded)

### Asserting a panic in host-tested, hardware-independent validation logic

```
struct ChannelId(u8);

impl ChannelId {
    fn new(value: u8) -> Self {
        assert!(value < 8, "ADC channel {value} out of range (0..=7)");
        ChannelId(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_channel() {
        assert_eq!(ChannelId::new(3).0, 3);
    }

    #[test]
    #[should_panic(expected = "out of range")] // <- runs on the host; no on-target equivalent exists
    fn rejects_an_out_of_range_channel() {
        ChannelId::new(9);
    }
}
```

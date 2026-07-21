---
title: "#[should_panic]"
kind: attribute
embedded_support: partial
groups: ["Testing & Tooling"]
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

## Basic usage example

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

## Best practices & deeper information

### Scenario: Testing

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

**Why this way:** without `expected`, `rejects_zero_quantity` would also
"pass" if `PositiveQuantity::new` panicked for a completely unrelated
reason — an integer overflow elsewhere in the function, say — masking a
real bug behind a green test; the
[Rust Book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#checking-for-panics-with-should_panic)
recommends the `expected` argument specifically so a `#[should_panic]`
test verifies the *intended* failure, not merely that some panic occurred.

## Embedded Rust Notes

**Partial support.** `#[should_panic]` is a modifier on `#[test]`, which
needs `std` for the host-run test harness that catches the panic and
reports pass/fail — see [`#[test]`](test-attribute.md)'s Embedded Rust
Notes. Host-tested, hardware-independent logic from a `#![no_std]` crate
(a packet parser rejecting a malformed frame, say) can still use
`#[should_panic]` normally when tested on the host toolchain; it has no
on-target equivalent in a bare-metal test harness like `defmt-test`.

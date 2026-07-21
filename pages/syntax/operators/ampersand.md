---
title: "&"
kind: operator
embedded_support: full
groups: [Bitwise, Basics, "Ownership & Borrowing"]
related_concepts: ["Borrowing (shared references)", "Mutable borrowing", Operator overloading]
related_syntax: [mut, "*", "&&", "&="]
see_also: ["*", mut]
---

## Explanation

`&` has two unrelated meanings, separated by position:

1. **Prefix: borrow.** `&expr` produces a shared reference to `expr`
   without taking ownership of it; `&type` (as in `&i32`, `&'a str`) is
   the *type* of such a reference. This is the far more common use in
   everyday Rust code, and is covered in depth on the Borrowing concept
   page — the syntax angle is just: `&` creates a reference, `*`
   (see [`*`](asterisk.md)) follows one back to its target.
2. **Binary: bitwise AND.** `a & b` between two integers, overloadable via
   `std::ops::BitAnd` (`BitAnd` is also implemented for `bool`, making
   `a & b` a non-short-circuiting logical AND — relevant when choosing
   between `&&` and `&`). Also appears in trait-bound-adjacent contexts as
   part of `&` reference types combined with lifetimes: `&'a mut T`.

`&mut expr` / `&mut Type` is the mutable-borrow counterpart, but it is
its own two-keyword combination rather than a separate single token —
see [`mut`](../keywords/mut.md).

`&&` is a distinct token (see [`&&`](ampersand-ampersand.md)), not two
`&` read together, though `&&expr` (a reference to a reference) is valid —
the lexer still produces the single `&&` token there, and it's the
*parser* that splits it into two `&`s when it appears in borrow position.

## Usage examples

### Creating a shared reference

```
let x = 5;
let r = &x; // <- `&` borrows `x`, producing a shared reference `&i32`
```

### Sharing data with multiple references

Several parts of a program often need to read the same value without any
of them taking ownership of it — `&` lets each function borrow it
independently.

```
struct Config {
    name: String,
    max_connections: u32,
}

fn print_summary(config: &Config) { // <- `&Config` borrows instead of taking ownership
    println!("{}: {} conns", config.name, config.max_connections);
}

fn is_valid(config: &Config) -> bool { // <- a second, simultaneous shared borrow
    config.max_connections > 0
}

let config = Config { name: "primary".into(), max_connections: 10 };
print_summary(&config);
if is_valid(&config) {
    println!("ready");
}
```

Because `&T` is read-only, any number of shared borrows
can coexist safely — per [the Book's borrowing chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
this is what lets Rust share data across a program without cloning it
just to satisfy the borrow checker.

### Multi-threading

`std::thread::scope` lets spawned threads borrow data from the enclosing
stack frame with plain `&`, instead of requiring `Arc` and `move`,
because the scope guarantees every thread finishes before the borrow
ends.

```
let readings = vec![21.5, 22.0, 21.8, 23.1];

std::thread::scope(|s| {
    s.spawn(|| { // <- this closure borrows `&readings`, not an owned copy
        let avg: f64 = readings.iter().sum::<f64>() / readings.len() as f64;
        println!("average: {avg}");
    });
    s.spawn(|| { // <- a second thread borrowing the same `readings` at once
        let max = readings.iter().cloned().fold(f64::MIN, f64::max);
        println!("max: {max}");
    });
});
```

[`std::thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html)
statically proves the spawned threads can't outlive `readings`, so the
compiler accepts a plain `&` borrow here where a non-scoped `thread::spawn`
would demand `'static` data (typically via `Arc`).

### Designing a public API

A function that only needs to read its argument should accept `&T`
rather than an owned `T` — taking ownership needlessly forces every
caller to give up or clone their value.

```
struct Order {
    id: u32,
    items: Vec<String>,
}

// PREFER: a shared borrow lets the caller keep using `order` afterward
fn total_items(order: &Order) -> usize { // <- `&Order`, not `Order`
    order.items.len()
}

// AVOID: taking `Order` by value forces callers to clone or give it up
fn total_items_owned(order: Order) -> usize {
    order.items.len()
}
```

The [API Guidelines' flexibility guidance](https://rust-lang.github.io/api-guidelines/flexibility.html)
recommends borrowing over owning in function signatures whenever the
function doesn't need to store or consume the value, since it's strictly
less restrictive for every caller.

## Explanation (Embedded)

Both meanings of `&` carry over unchanged in `#![no_std]` firmware, and
both are used constantly. The borrow meaning is how driver code hands
out access to a peripheral without giving it up — a HAL function that
only reads a peripheral's state typically takes `&Peripheral` rather
than consuming it, exactly as in hosted code, and the borrow checker
still enforces that no other code holds a conflicting `&mut` to the
same peripheral at the same time, which is exactly how embedded Rust
statically prevents two pieces of code from fighting over one register
block. The bitwise meaning is how register state is *tested*: reading
a status register into a plain integer and combining it with `&`
against a bitmask asks "is this particular flag set?" without touching
the register — setting or clearing bits is `|=`/`&=` (their own pages),
and plain `&` never writes anything back. Because `&` never mutates,
testing a flag this way is safe to repeat freely, including from
inside an interrupt handler that only needs to read a status bit, not
acknowledge it.

## Usage examples (Embedded)

### Testing a status flag before writing to a peripheral

```
const USART_SR: *const u32 = 0x4001_3800 as *const u32; // USART1 status register
const TXE: u32 = 1 << 7; // transmit-data-register-empty flag

fn tx_ready() -> bool {
    let status = unsafe { core::ptr::read_volatile(USART_SR) };
    status & TXE != 0 // <- `&` tests the TXE bit; it only reads, it never writes
}
```

### Borrowing a peripheral handle in driver code

```
struct Gpio {
    idr: *const u32, // input data register
}

fn read_pin(gpio: &Gpio, pin: u8) -> bool { // <- `&Gpio` borrows the handle instead of consuming it
    let bits = unsafe { core::ptr::read_volatile(gpio.idr) };
    bits & (1 << pin) != 0
}

let gpio = Gpio { idr: 0x4001_0810 as *const u32 };
let pressed = read_pin(&gpio, 5); // <- a second call could borrow `&gpio` again at the same time
```

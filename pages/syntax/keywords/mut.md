---
title: "mut"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Ownership, Borrowing (shared references), Mutable borrowing]
related_syntax: [let, "&"]
see_also: [let, "&"]
---

## Explanation

`mut` marks a binding, reference, or raw pointer as mutable — the one
thing in Rust that is *not* mutable by default. It appears in a few
distinct syntactic positions that are easy to conflate:

- **On a `let` binding:** `let mut x = 5;` allows `x` itself to be
  reassigned or mutated later. Without `mut`, `x = 6;` is a compile error.
- **On a reference type:** `&mut T` is a mutable (exclusive) reference —
  a different type from `&T`, not a modifier of it. Only one `&mut T` to a
  given value can exist at a time, and it cannot coexist with any `&T`.
- **On a function parameter pattern:** `fn f(mut x: i32)` makes the
  parameter binding mutable inside the function body — this is purely
  local; it says nothing about the caller's variable and has no effect on
  the function's signature/type.
- **On `self`:** `&mut self` in a method signature borrows the receiver
  mutably.

`mut` is not part of a type in the `let mut x` sense (the binding is
mutable, not the type `i32`), but it *is* part of the type in the
reference sense (`&mut T` and `&T` are different types entirely).

## Usage examples

### Making a binding mutable with `let mut`

```
let mut x = 5; // <- `mut` allows `x` to be reassigned
x = 6;
```

**Restriction:** `mut` must appear at the binding site (`let mut x`); it
cannot be added later to make an already-immutable binding mutable.

### Sharing state across threads

`mut` disappears from the signature when state is shared across threads —
the mutability moves into the lock, and the `Arc` binding itself stays
immutable even though what it points to is mutated from multiple threads.

```
use std::sync::{Arc, Mutex};
use std::thread;

let readings = Arc::new(Mutex::new(Vec::new())); // note: no `mut` — the Mutex supplies the mutability
let mut handles = Vec::new(); // <- `mut` needed here: the Vec of handles is grown with `push`

for sensor_id in 0..4 {
    let readings = Arc::clone(&readings);
    handles.push(thread::spawn(move || {
        let mut batch = readings.lock().unwrap(); // <- `mut` binds the guard for write access
        batch.push(sensor_id as f64 * 1.5);
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

Declaring the `Arc` binding `mut` would be misleading —
interior mutability means the binding is never reassigned; Clippy flags
unneeded `mut` bindings via
[`unused_mut`](https://doc.rust-lang.org/rustc/lints/listing/warn-by-default.html#unused-mut).

### Modifying an existing object

An object with invariants to protect exposes a `&mut self` method rather
than a public field a caller could set to any value directly.

```
struct Order { total_cents: u64, shipped: bool }

impl Order {
    fn mark_shipped(&mut self) {
        // <- `&mut self` lets the method mutate the receiver in place
        self.shipped = true;
    }
}

let mut order = Order { total_cents: 4200, shipped: false }; // <- `mut` needed: `order` is mutated below
order.mark_shipped();
```

Routing mutation through a method rather than a public
field keeps the invariant ("shipped only goes from false to true") in one
place — the
[Book's method-syntax chapter](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
covers `&mut self` as the standard way to expose in-place mutation.

### Mutating through a reference

Doubling every price in a slice in place needs a mutable reference to
each element, not just a mutable binding to the slice itself.

```
fn double_all(values: &mut [f64]) {
    for value in values.iter_mut() {
        *value *= 2.0; // <- `mut` reference lets us write through `value`, not just read it
    }
}

let mut prices = [9.99, 14.50, 3.25]; // <- `mut` required: `double_all` needs a mutable borrow
double_all(&mut prices);
```

[`iter_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.iter_mut)
is the standard way to mutate every element of a slice in place without
manual indexing, and the borrow checker guarantees only one `&mut`
exists at a time, ruling out aliasing bugs at compile time.

## Explanation (Embedded)

`mut` means exactly the same thing under `#![no_std]` as it does on a
hosted target — a binding-mutability marker, a distinct reference type
(`&mut T`), and the standard way a method takes exclusive access to
`self`. It shows up constantly in embedded code because so much of a
HAL's public API is built around `&mut self` methods: setting a GPIO
pin, writing a UART byte, or reconfiguring a timer's prescaler all take
`&mut self` on the peripheral handle, since the operation changes
hardware state, and the borrow checker's "only one `&mut` at a time"
rule is exactly what stops two parts of the program from racing to
reconfigure the same peripheral through two live handles at once.

A second, historically important pattern is `static mut` — a global
marked mutable so an interrupt handler and the main loop can both reach
the same piece of state (a tick counter, a ring-buffer index) with no
owning stack frame to route the borrow through. Every access to a
`static mut` requires an `unsafe` block, because the compiler cannot
verify that the interrupt and the main loop won't touch it at the same
instant with no synchronization — the same data race an ordinary
`&mut`/`&` aliasing rule would rule out at compile time if the state
weren't global. `static mut` still appears throughout older and simpler
embedded codebases, but it's increasingly superseded by a `static`
wrapping a `critical-section`-guarded primitive (`Mutex<Cell<_>>`), which
gets the same interrupt-shared mutability without an `unsafe` block at
every access site — see [`static`](static.md) for that pattern in depth.

## Usage examples (Embedded)

### Mutating a peripheral through `&mut self`

```
use embedded_hal::digital::OutputPin;

fn blink<P: OutputPin>(led: &mut P) { // <- `&mut P`: exclusive access needed to change the pin's output level
    led.set_high().ok();
    led.set_low().ok();
}
```

### `static mut` as interrupt-shared state (historical pattern)

```
static mut TICKS: u32 = 0; // <- `mut`: shared between `main` and the interrupt, so every access needs `unsafe`

#[interrupt]
fn SysTick() {
    unsafe {
        TICKS += 1; // <- `mut` access requires `unsafe`: the compiler can't verify no data race with `main`
    }
}

fn uptime_ticks() -> u32 {
    unsafe { TICKS } // <- the same `unsafe` requirement applies to reads, not just writes
}
```

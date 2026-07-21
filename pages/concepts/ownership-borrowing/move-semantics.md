---
title: "Move semantics"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Move Semantics", "Unique to Rust", "Coming from C / C++"]
related_syntax: [move, "="]
see_also: ["Ownership", "Copy vs Clone"]
---

## Explanation

Assigning a value to a new variable, passing it to a function, or
returning it from one transfers ownership rather than copying it by
default — this is a **move**. After a move, the original binding is no
longer valid; the compiler tracks this and rejects any later use of it as
a compile error, not a runtime bug.

This is a deliberate departure from two more familiar defaults: it's not
implicit reference/pointer semantics (as in Python, Java, JS, where
assignment shares the same object and mutation is visible through every
reference to it), and it's not implicit copying (as in C++, where
`Foo b = a;` invokes a copy constructor by default, unless you explicitly
write `std::move(a)`). Rust flips the C++ default: moving is the norm,
and copying only happens when a type explicitly opts in via `Copy` (see
[Copy vs Clone](copy-vs-clone.md)) or you call `.clone()` yourself.

The benefit is that "who owns this, and is it still valid here" is always
statically knowable and enforced by the compiler — there's no way to
accidentally hold onto and use a value that's already been logically
handed off elsewhere, a whole category of bug (use-after-move,
double-free) that move semantics eliminates by construction rather than
by convention or discipline.

## Basic usage example

```
let a = String::from("hi");
let b = a; // <- ownership moves from `a` to `b`
// using `a` here is a compile error: value was moved
```

## Best practices & deeper information

### Scenario: Transferring ownership

A function that consumes a `String` to build a `Report` should take it by
value, not by reference — the report becomes the text's new owner, and
the signature says so.

```
struct Report {
    body: String,
}

fn finalize(body: String) -> Report { // <- takes ownership: `body` is moved in, not borrowed
    Report { body }
}

let draft = String::from("quarterly summary");
let report = finalize(draft); // <- `draft` moves here
// println!("{draft}"); // would fail to compile: draft was moved into finalize
println!("{}", report.body);
```

**Why this way:** taking `String` by value instead of `&str` is the right
signature when the callee needs to keep the data — moving avoids an
unnecessary clone, and the compiler statically stops the caller from
reusing a value it no longer owns, which the
[Rust Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
highlights as move semantics' main benefit over languages with implicit
sharing.

### Scenario: Multi-threading

Spawning a worker thread that might outlive the function that started it
means the thread needs to own its input data for its whole lifetime, not
just borrow it.

```
use std::thread;

let batch = vec![1, 2, 3, 4, 5];

let handle = thread::spawn(move || { // <- `move` forces `batch` to move into the closure, not borrow it
    let sum: i32 = batch.iter().sum();
    println!("batch sum: {sum}");
});

// batch is no longer usable here: ownership moved into the spawned thread
handle.join().unwrap();
```

**Why this way:** `thread::spawn` can't prove the spawned thread finishes
before the caller's local variables go out of scope, so it requires
`'static` data — moving ownership into the closure with `move` is how a
value that isn't already `'static` becomes safe to hand to an
independently-running thread; the
[Rust Book](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
covers this as the standard way to share owned data with a spawned
thread.

## Explanation (Embedded)

Move semantics are core-language: the compiler's move-vs-copy tracking has
nothing to do with an allocator, a runtime, or `std`, so the mechanism
described above is byte-for-byte identical under `#![no_std]`. What
changes is what gets moved. Embedded code moves ownership of things that
don't exist in most hosted programs — a peripheral handle, a GPIO pin, a
DMA channel — and the same use-after-move rule that stops a hosted program
from reusing a moved `String` is what stops firmware from reusing a
peripheral handle it has already handed off to a driver.

The most common embedded move is into a driver's constructor: a HAL type
like `Usart` or `Spi` takes ownership of the raw peripheral/pin types it
wraps, so once `Usart::new(usart1, tx_pin, rx_pin)` runs, `usart1` is gone
from the caller's scope — there is no way to accidentally reconfigure the
same UART directly through the register block while the driver also
believes it owns it. The second common case is a `move` closure or task
function capturing a peripheral so it can run independently of the code
that set it up — an interrupt handler registered through a framework like
RTIC, or an `async` task spawned on an executor like `embassy` — which
needs owned, `'static` data for exactly the reason `thread::spawn` does on
a hosted target: the closure/task may still be running long after the
function that created it has returned.

## Basic usage example (Embedded)

```
struct Led { pin: GpioPin }

impl Led {
    fn new(pin: GpioPin) -> Self { // <- takes ownership: the caller can no longer touch `pin` directly
        Led { pin }
    }
}

let pin = GpioPin::new(5);
let led = Led::new(pin); // <- `pin` moves into `led`; using `pin` afterward is a compile error
```

## Best practices & deeper information (Embedded)

### Scenario: Transferring ownership

A UART driver should take ownership of the raw peripheral it wraps in its
constructor, so nothing else in the program can reconfigure the same
registers behind the driver's back.

```
struct RawUsart1; // stand-in for a PAC-generated peripheral type
struct Usart { inner: RawUsart1 }

impl Usart {
    fn new(usart1: RawUsart1) -> Self { // <- ownership moves in: usart1 is now exclusively the driver's
        // ... configure baud rate, enable the peripheral clock, etc.
        Usart { inner: usart1 }
    }

    fn write_byte(&mut self, byte: u8) {
        let _ = (&self.inner, byte); // placeholder for a real register write
    }
}

let usart1 = RawUsart1;
let mut usart = Usart::new(usart1); // <- usart1 moved; no other code can construct a second Usart from it
usart.write_byte(b'A');
```

**Why this way:** moving the raw peripheral into the driver's constructor
means the type system — not a runtime check or a comment — guarantees
there's exactly one `Usart` wrapping this hardware; the
[Rust Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)'s
"one owner at a time" rule is precisely what makes a second, conflicting
`Usart::new(usart1)` a compile error rather than a runtime race over the
same registers.

### Scenario: Multi-threading

Spawning an `embassy` async task to drive a sensor independently of
`main` needs the task to own its peripheral handle outright, the same way
`thread::spawn` needs owned, `'static` data on a hosted target.

```
struct AdcChannel; // stand-in for a HAL ADC handle

#[embassy_executor::task]
async fn sample_task(mut adc: AdcChannel) { // <- task owns `adc` for as long as it runs, independent of main
    loop {
        // ... await a conversion, publish the reading
        let _ = &mut adc;
    }
}

fn spawn_sampling(spawner: embassy_executor::Spawner, adc: AdcChannel) {
    spawner.spawn(sample_task(adc)).unwrap(); // <- `adc` moves into the task here
    // using `adc` in `main` after this point is a compile error
}
```

**Why this way:** an async task or interrupt handler can outlive the
function that set it up, so it can't safely hold a borrow into that
function's stack frame — moving ownership in is what makes the peripheral
handle valid for the task's entire, independently-scheduled lifetime,
mirroring the
[Rust Book's `move` closure guidance for `thread::spawn`](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
applied to an executor instead of an OS thread.

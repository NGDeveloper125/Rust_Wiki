---
title: "Return consumed argument on error"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Handling Errors & Failure"]
related_syntax: []
see_also: ["Result<T, E>", "Custom error types", "mem::take / mem::replace"]
---

## Explanation

A function that takes an argument by value — consuming it, per normal
[move semantics](../ownership-borrowing/move-semantics.md) — has a
problem when it can fail partway through: if it returns only an error,
the argument it consumed is gone for good, even though the operation
never actually completed. The caller is left having to reconstruct or
re-obtain a value it already had a perfectly good copy of a moment
earlier, for no reason other than that the failing function happened to
take ownership of it.

The idiom's fix is to design the error type so the original argument
rides along inside it: instead of `fn configure(name: String) ->
Result<Config, ConfigError>`, the signature becomes `fn configure(name:
String) -> Result<Config, (ConfigError, String)>` (or an error struct
with a named field holding the string). On failure, the caller gets both
*why* it failed and the exact value it handed over, ready to inspect,
log, retry with, or fix and resubmit — without ever needing to keep its
own separate copy around just in case.

This is exactly the shape the standard library uses for channel sends:
`Sender::send` takes its argument by value, and if the receiving end has
disconnected, it returns `Err(SendError<T>)` — a type that wraps the
original value right back to the caller, since a `Sender` that quietly
dropped an unsent message on failure would be a data-loss trap. The same
logic applies to any consuming operation where failure is a real,
expected possibility: a queue that's full, a socket that's closed, a
validation step that rejects the value it was handed.

The alternative of taking the argument by reference (`&self` or `&T`)
instead of by value sidesteps the whole problem when it's available —
the caller never loses ownership in the first place, so there's nothing
to hand back. Return-consumed-argument-on-error is specifically for the
cases where taking ownership genuinely is the right design (the function
wants to store the value on success, or transform it in place) but
failure still needs to be non-destructive.

## Basic usage example

```
struct ConfigError(String); // the failure reason

fn set_hostname(name: String) -> Result<String, (ConfigError, String)> {
    if name.is_empty() {
        return Err((ConfigError("hostname cannot be empty".to_string()), name)); // <- returns the argument back on failure
    }
    Ok(name)
}

match set_hostname(String::new()) {
    Ok(name) => println!("accepted: {name}"),
    Err((err, returned)) => println!("{}: got {:?} back", err.0, returned), // <- caller still owns the original String
}
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

A message queue's `enqueue` consumes the message it's given, but if the
queue is full, the caller shouldn't lose the message it was trying to
send — the error carries it back out.

```
struct Order {
    id: u64,
}

struct QueueFullError {
    limit: usize,
}

struct OrderQueue {
    items: Vec<Order>,
    limit: usize,
}

impl OrderQueue {
    fn enqueue(&mut self, order: Order) -> Result<(), (QueueFullError, Order)> {
        if self.items.len() >= self.limit {
            return Err((QueueFullError { limit: self.limit }, order)); // <- `order` comes back, not lost
        }
        self.items.push(order);
        Ok(())
    }
}

let mut queue = OrderQueue { items: Vec::new(), limit: 0 };
match queue.enqueue(Order { id: 42 }) {
    Ok(()) => println!("enqueued"),
    Err((_err, order)) => println!("queue full, order {} was not lost", order.id),
}
```

**Why this way:** a caller that just spent effort building an `Order`
shouldn't have to rebuild it after a failed `enqueue` just because the
queue happened to take ownership — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/return-consumed-arg-on-error.html)
book documents returning the consumed argument inside the error as the
idiomatic way to make a consuming operation's failure non-destructive.

### Scenario: Designing a public API

Modeling a channel-like `send` after the standard library's own
`Sender::send` means shaping the error type as a wrapper that hands the
unsent value straight back, matching a pattern callers already know from
`std::sync::mpsc`.

```
struct SendError<T>(pub T); // <- mirrors std::sync::mpsc::SendError<T>'s shape

struct EventBus<T> {
    subscribers: usize,
    buffer: Vec<T>,
}

impl<T> EventBus<T> {
    fn send(&mut self, event: T) -> Result<(), SendError<T>> {
        if self.subscribers == 0 {
            return Err(SendError(event)); // <- caller gets `event` back, not just a failure signal
        }
        self.buffer.push(event);
        Ok(())
    }
}

let mut bus: EventBus<String> = EventBus { subscribers: 0, buffer: Vec::new() };
if let Err(SendError(event)) = bus.send("shutdown".to_string()) {
    println!("no subscribers; recovered event: {event}");
}
```

**Why this way:** following the same `SendError<T>`-style shape the
standard library already uses for
[`mpsc::Sender::send`](https://doc.rust-lang.org/std/sync/mpsc/struct.SendError.html)
gives callers a pattern they can recognize immediately, and it's the
concrete precedent the return-consumed-argument idiom is modeled on.

## Explanation (Embedded)

The non-destructive-failure argument applies with extra force on
embedded targets, where the value a fallible call consumes is often a
buffer that was genuinely expensive to fill — samples pulled from an
ADC, a frame assembled for DMA — and where there may be no cheap way to
"just get another one" once it's gone. A peripheral-transaction API, like
an SPI or I2C driver's `write`/`transfer`, takes its outgoing buffer by
value so it can hold it for the duration of the transaction (and, on some
HALs, hand it straight to a DMA engine); a failed transaction — a NACK
from the target device, a bus timeout, lost arbitration — shouldn't also
cost the caller its data. Shaping the error as `Err((Error, Buf))`, the
same shape `std::sync::mpsc::SendError<T>` uses, lets the caller retry
the exact bytes, log them, or fall back to a bit-banged implementation
without re-populating the buffer from scratch.

## Basic usage example (Embedded)

```
struct I2cError; // NACK, bus timeout, arbitration loss, ...

fn i2c_write(addr: u8, buf: [u8; 4]) -> Result<(), (I2cError, [u8; 4])> {
    if addr == 0 {
        return Err((I2cError, buf)); // <- caller gets the un-sent buffer back, not just a failure
    }
    Ok(())
}

match i2c_write(0, [1, 2, 3, 4]) {
    Ok(()) => {}
    Err((_err, buf)) => {
        let _ = buf; // still owned: can retry, log, or fall back without rebuilding it
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Handling and propagating errors

An SPI driver's `write` takes ownership of a fixed-size frame built up
from ADC samples so it can be handed straight to a DMA transfer; if the
bus reports a timeout, the caller shouldn't have to re-sample the ADC
just to retry the same frame.

```
struct SpiError; // bus timeout, mode fault, ...

struct SpiBus;

impl SpiBus {
    fn write(&mut self, frame: [u8; 8]) -> Result<(), (SpiError, [u8; 8])> {
        let bus_busy = true; // stand-in for a real bus-status register check
        if bus_busy {
            return Err((SpiError, frame)); // <- `frame` comes back, not lost with the failed transaction
        }
        Ok(())
    }
}

let mut spi = SpiBus;
match spi.write([0xAA; 8]) {
    Ok(()) => {}
    Err((_err, frame)) => {
        let _ = spi.write(frame); // <- retries the same frame instead of re-reading the ADC
    }
}
```

**Why this way:** re-acquiring the data behind a failed peripheral
transaction can mean re-sampling a sensor or re-running a DMA setup —
real cost on hardware, not just inconvenience — so handing the buffer
back inside the error, the shape the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/return-consumed-arg-on-error.html)
book documents for `Sender::send`, is what makes retrying cheap.

### Scenario: Designing a public API

A HAL author modeling a `write` method's error type after the same
convention `std::sync::mpsc` uses shapes it as a struct carrying the
original bytes back, so firmware crates get a retry pattern they already
recognize from channel APIs.

```
pub struct WriteError<B>(pub B); // <- mirrors std::sync::mpsc::SendError<T>'s shape, for a HAL transaction

pub struct Uart;

impl Uart {
    pub fn write(&mut self, bytes: [u8; 16]) -> Result<(), WriteError<[u8; 16]>> {
        let tx_fifo_full = false; // stand-in for a real hardware status flag
        if tx_fifo_full {
            return Err(WriteError(bytes)); // <- caller gets the bytes back, not just an error signal
        }
        Ok(())
    }
}
```

**Why this way:** giving a HAL's fallible transaction methods the same
`SendError<T>`-style shape the standard library already uses keeps error
handling non-destructive even though the underlying transport is a raw
register interface, not a channel, and lets firmware authors recognize
the retry pattern immediately.

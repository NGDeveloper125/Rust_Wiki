---
title: "Mutable borrowing"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["&", mut]
see_also: ["Borrowing (shared references)", "The borrow checker", "Interior mutability (Cell & RefCell)"]
---

## Explanation

A mutable reference (`&mut T`) grants temporary, exclusive write access to
a value without transferring ownership. Unlike a shared reference, only
one `&mut T` to a given value can exist at a time, and while it exists, no
other reference — shared or mutable — to that same value may exist
alongside it.

This exclusivity is the mechanism that rules out data races at compile
time: two threads (or even two pieces of code on the same thread) can
never simultaneously read-while-writing or write-while-writing the same
data, because the compiler statically guarantees a `&mut T` is always
alone. This rule is often summarized as "aliasing XOR mutability" — a
value can have multiple readers *or* one writer, never both at once.

The restriction can feel strict when you genuinely need shared, mutable
access (a cache multiple parts of a program update, a graph with back
edges) — that's precisely the gap
[interior mutability](interior-mutability.md) (`Cell`/`RefCell`, and their
thread-safe counterparts `Mutex`/`RwLock`) exists to fill, by moving the
exclusivity check from compile time to run time in a controlled way.

## Basic usage example

```
let mut v = vec![1, 2, 3];
let r = &mut v; // <- exclusive: no other reference to v may exist while r is alive
r.push(4);
println!("{r:?}");
```

**Restriction:** while `r` is alive, not even the owner (`v`) can be read
or written through another path — this is enforced at compile time.

## Best practices & deeper information

### Scenario: Mutating through a reference

A function that normalizes sensor readings in place only needs write
access, not custody of the collection — `&mut [f32]` says so directly,
instead of taking and returning ownership of the whole `Vec`.

```
fn clamp_readings(readings: &mut [f32], max: f32) {
    for r in readings.iter_mut() { // <- `&mut` lets each element be updated in place
        if *r > max {
            *r = max;
        }
    }
}

let mut readings = vec![12.4, 99.9, 3.1];
clamp_readings(&mut readings, 50.0);
println!("{readings:?}");
```

**Why this way:** taking `&mut [f32]` instead of `Vec<f32>` by value lets
the caller keep ownership of the collection and avoids an unnecessary
move — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html)
favor the signature that asks for the least access a function actually
needs.

### Scenario: Modifying an existing object

Updating a user's profile through a method that takes `&mut self` keeps
related fields consistent with each other, instead of relying on every
call site to update several public fields correctly on its own.

```
struct UserProfile {
    display_name: String,
    login_count: u32,
}

impl UserProfile {
    fn record_login(&mut self, name: Option<&str>) { // <- exclusive access needed to update both fields together
        if let Some(name) = name {
            self.display_name = name.to_string();
        }
        self.login_count += 1;
    }
}

let mut user = UserProfile { display_name: "guest".into(), login_count: 0 };
user.record_login(Some("nimrod"));
```

**Why this way:** grouping related mutations behind one `&mut self`
method keeps the struct's invariants (here, `login_count` always
reflecting a login) enforced in one place — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/index.html)
book generally favors encapsulated mutation over exposing raw mutable
fields that every caller has to update correctly by hand.

## Explanation (Embedded)

`&mut T`'s exclusivity rule needs no allocator and no `std` — it's core
language, so it applies exactly as written above under `#![no_std]`. What
makes it especially load-bearing in embedded code is what the "T" usually
is: a HAL driver wrapping a peripheral. A method like
`fn write(&mut self, byte: u8)` on an I2C or SPI driver isn't just an API
convention — it's the compiler's guarantee that no other code anywhere in
the program can be simultaneously mid-transaction with that same bus.
Two `&mut` borrows of the same peripheral can never coexist, so it's
impossible to compile code where, say, an interrupt handler and the main
loop both believe they're free to issue the next byte of an I2C transfer
at the same time.

That guarantee is compile-time and free of runtime cost — no lock to
acquire, no mutex to poison — which matters on hardware with no OS to fall
back on for scheduling or fairness. It's also why frameworks like RTIC
build their whole resource-sharing model on top of `&mut` access: a
resource declared local to one task is handed out as `&mut` exactly once
per access window, so the same aliasing-XOR-mutability rule that stops two
closures from both mutating a `Vec` is what stops a main loop and an
interrupt from both mid-transaction with the same peripheral.

## Basic usage example (Embedded)

```
struct I2cBus; // stand-in for a HAL I2C peripheral handle

impl I2cBus {
    fn write(&mut self, addr: u8, byte: u8) { // <- exclusive access: no other code can also be mid-transaction
        let _ = (addr, byte); // placeholder for the real register write
    }
}

fn set_sensor_gain(bus: &mut I2cBus, addr: u8, gain: u8) {
    bus.write(addr, gain);
}
```

## Best practices & deeper information (Embedded)

### Scenario: Mutating through a reference

A multi-byte I2C write needs exclusive access to the bus for its entire
duration — taking `&mut self` on the driver method is what stops any other
code from interleaving a conflicting transaction in the middle of it.

```
struct I2cBus; // stand-in for a HAL I2C peripheral handle

impl I2cBus {
    fn write_register(&mut self, addr: u8, reg: u8, value: u8) { // <- &mut self: exclusive for the whole transfer
        let _ = self.start(addr);
        let _ = self.send(reg);
        let _ = self.send(value);
        self.stop();
    }

    fn start(&mut self, addr: u8) -> bool { let _ = addr; true }
    fn send(&mut self, byte: u8) -> bool { let _ = byte; true }
    fn stop(&mut self) {}
}

let mut bus = I2cBus;
bus.write_register(0x68, 0x1B, 0x07); // <- one exclusive borrow covers start+send+send+stop
```

**Why this way:** if `send`/`stop` instead took `&self` and mutated the
bus through, say, an unsafe register pointer, nothing would stop two
independent call sites from interleaving their bytes on the wire — taking
`&mut self` for the whole `write_register` call means the borrow checker
itself enforces "one complete transaction at a time," matching how the
[embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/) traits
model bus access as `&mut self` methods throughout.

### Scenario: Modifying an existing object

Configuring several related fields of a GPIO pin together — mode, speed,
pull resistor — through one `&mut self` method keeps them consistent,
instead of exposing public fields a caller could update out of order.

```
struct GpioPin { mode: u8, pull_up: bool }

impl GpioPin {
    fn configure_as_output(&mut self) { // <- exclusive access to update both fields together
        self.mode = 0b01; // output mode
        self.pull_up = false; // outputs don't need a pull resistor
    }
}

let mut led_pin = GpioPin { mode: 0, pull_up: true };
led_pin.configure_as_output();
```

**Why this way:** grouping the register-level fields behind one `&mut
self` method keeps the pin's configuration internally consistent — the
[API Guidelines](https://rust-lang.github.io/api-guidelines/flexibility.html)
preference for methods over raw mutable fields applies just as much to a
struct that happens to shadow a hardware register layout as to any other
type.

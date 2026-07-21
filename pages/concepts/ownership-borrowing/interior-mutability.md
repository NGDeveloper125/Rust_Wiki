---
title: "Interior mutability (Cell & RefCell)"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Interior Mutability", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Mutable borrowing", "The borrow checker", "Shared ownership (Rc & Arc)"]
---

## Explanation

Interior mutability lets you mutate a value through a shared (`&T`)
reference — something the borrow checker otherwise forbids outright — by
moving the exclusivity check from compile time to run time.

`Cell<T>` allows getting and setting a `Copy` value through a shared
reference with no runtime check at all (it never hands out a reference to
the inner value, only whole-value copies in and out, so there's nothing
to check). `RefCell<T>` goes further, handing out actual `&T`/`&mut T`
borrows of its contents on demand, but tracks how many are outstanding at
runtime and panics if the "aliasing XOR mutability" rule would be
violated — the same rule the compiler enforces statically for ordinary
references, just deferred to when the program actually runs.

This exists for the real cases where the compiler's static analysis is
too conservative to accept a genuinely safe pattern: a struct that needs
to update an internal cache from behind a shared reference, or a graph
structure with cyclic references. It's frequently paired with
[`Rc`/`Arc`](shared-ownership-rc-arc.md) (`Rc<RefCell<T>>` is a very
common combination) since shared ownership alone only grants shared
*reference* access — interior mutability is what makes that shared access
mutable too. The cost of this flexibility is that a logic error (two
overlapping mutable borrows of a `RefCell`) becomes a runtime panic
instead of a compile-time error — the safety guarantee is preserved, but
enforcement moves later, and with it the chance of catching the mistake.

## Basic usage example

```
use std::cell::RefCell;

let data = RefCell::new(5);
*data.borrow_mut() += 1; // <- mutates through a shared RefCell value
println!("{}", data.borrow());
```

**Restriction:** `RefCell`'s borrow rules are checked at runtime, not
compile time — holding two overlapping `borrow_mut()`s (or a `borrow()`
alongside a `borrow_mut()`) panics instead of failing to compile.

## Best practices & deeper information

### Scenario: Modifying an existing object

A single-threaded lookup type caches its last computed result behind a
`RefCell`, so `&self` methods can still update the cache internally
without forcing every caller through `&mut self`.

```
use std::cell::RefCell;

struct PriceLookup {
    cache: RefCell<Option<(String, f64)>>,
}

impl PriceLookup {
    fn price_for(&self, sku: &str) -> f64 { // <- `&self`, not `&mut self`: mutation stays internal
        if let Some((cached_sku, price)) = &*self.cache.borrow() {
            if cached_sku == sku {
                return *price;
            }
        }
        let price = 19.99; // pretend this is an expensive lookup
        *self.cache.borrow_mut() = Some((sku.to_string(), price)); // <- mutates through &self
        price
    }
}
```

**Why this way:** `RefCell` lets a type present a read-only (`&self`)
public API while still updating internal bookkeeping like a cache — the
[Rust Book](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
frames this as the case interior mutability exists for: a mutation that's
an implementation detail, not part of the type's externally visible
contract.

### Scenario: Sharing state across threads

The moment that single-threaded cache needs to be touched from more than
one thread, `RefCell` stops being an option — the fix is `Mutex`, not a
thread-safe variant of `RefCell`.

```
use std::sync::Mutex;

struct SharedCounter {
    hits: Mutex<u64>, // <- Mutex, not RefCell: RefCell's borrow tracking isn't thread-safe
}

impl SharedCounter {
    fn record_hit(&self) {
        let mut hits = self.hits.lock().unwrap(); // <- blocks other threads instead of racing
        *hits += 1;
    }
}
```

**Why this way:** `RefCell` doesn't implement `Sync`, so it can't be
shared across threads at all — its runtime borrow check has no protection
against two threads racing on it simultaneously; `Mutex` is the
thread-safe equivalent, enforcing exclusivity by blocking instead of
panicking, as the
[Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers when introducing `Mutex<T>`.

## Explanation (Embedded)

`Cell` and `RefCell` both live in `core::cell`, so the mechanism described
above — moving the "aliasing XOR mutability" check from compile time to
runtime — works identically under `#![no_std]`, no allocator required.
`Cell<T>` is a particularly good fit for embedded code because so much
embedded state is naturally a small `Copy` value (a latest ADC sample, a
tick count, a status byte): `Cell::get`/`set` never hands out a reference
at all, so there's nothing to track and no runtime check to fail, making
it strictly cheaper than `RefCell` wherever the contained type is `Copy`.

Both types share one hard limitation that matters far more on embedded
than it typically does hosted: neither is `Sync` (see
[Send & Sync](../concurrency-async/send-and-sync.md)), so a bare
`Cell`/`RefCell` is not safe to reach from both `fn main`'s loop and a
`#[interrupt]` handler at once — the runtime check `RefCell` performs
assumes single-threaded access, and an interrupt preempting `main`
mid-borrow is exactly the kind of concurrent access that check was never
designed to catch. Embedded code's standard answer, since there's usually
no OS and therefore no `std::sync::Mutex` to fall back on, is
`critical_section::Mutex<Cell<T>>` or `critical_section::Mutex<RefCell<T>>`:
`critical-section`'s `Mutex` wrapper (a genuinely different type from
`std::sync::Mutex`, despite the shared name) is only `Sync` when its
contents are `Send`, and only permits touching the inner `Cell`/`RefCell`
from inside a critical section — typically interrupts-disabled on a
single-core target — which is what makes it sound to hand both `main` and
an interrupt handler a `'static` reference to the same cell. This is, in
effect, `no_std`'s substitute for `std::sync::Mutex<T>` in the one place
OS-level locking isn't available to fall back on.

## Basic usage example (Embedded)

```
use core::cell::Cell;

let sample = Cell::new(0u16);
sample.set(sample.get() + 1); // <- mutates through a shared Cell value, no runtime check needed: u16 is Copy
```

## Best practices & deeper information (Embedded)

### Scenario: Interior mutability

A driver caches the last-read calibration offset behind a `Cell` so its
`&self` reading method can update the cache without needing `&mut self` —
safe because the value is `Copy` and the driver never crosses an
interrupt boundary.

```
use core::cell::Cell;

struct Thermometer {
    last_offset: Cell<i16>, // <- Copy type: Cell is the cheapest form of interior mutability here
}

impl Thermometer {
    fn read_celsius(&self, raw: i16) -> i16 { // <- &self, not &mut self
        let offset = self.last_offset.get();
        let corrected = raw - offset;
        self.last_offset.set(corrected / 10); // <- mutates through &self, no reference ever handed out
        corrected
    }
}
```

**Why this way:** `Cell` never hands out a reference to its contents, so
there's no runtime borrow-tracking overhead at all — for a `Copy` value
like a calibration offset, it's strictly cheaper than `RefCell` while
giving the same "`&self` can still update internal state" capability the
classic `RefCell` cache example relies on.

### Scenario: Sharing state across threads

A tick counter incremented by a timer interrupt and read from the main
loop — the same no-OS pattern covered on
[Send & Sync](../concurrency-async/send-and-sync.md), here from the
interior-mutability angle: a bare `RefCell` isn't `Sync`, so it has to be
wrapped before an interrupt handler can touch it too.

```
use core::cell::RefCell;
use critical_section::Mutex;

static LAST_ERROR: Mutex<RefCell<Option<u8>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn USART1() {
    critical_section::with(|cs| {
        *LAST_ERROR.borrow_ref_mut(cs) = Some(read_status_register());
    });
}

fn main_loop() -> ! {
    loop {
        if let Some(code) = critical_section::with(|cs| LAST_ERROR.borrow_ref_mut(cs).take()) {
            handle_error(code);
        }
    }
}
```

**Why this way:** a bare `RefCell<Option<u8>>` isn't `Sync`, so `static
LAST_ERROR: RefCell<...>` would fail to compile the instant the interrupt
handler tried to touch it too — wrapping it in `critical-section`'s
`Mutex` is what makes the type `Sync` and routes every access through a
critical section, `no_std`'s equivalent of the `std::sync::Mutex` a hosted
program would reach for instead.

### Scenario: Modifying an existing object

A pressure sensor driver lazily computes and caches an expensive
linearization result behind `&self`, mirroring the classic single-threaded
cache pattern but grounded in a real embedded correction routine.

```
use core::cell::RefCell;

struct PressureSensor {
    cache: RefCell<Option<(u16, f32)>>, // (last raw reading, linearized result)
}

impl PressureSensor {
    fn read_kpa(&self, raw: u16) -> f32 { // <- &self: mutation of the cache stays an internal detail
        if let Some((cached_raw, kpa)) = *self.cache.borrow() {
            if cached_raw == raw {
                return kpa;
            }
        }
        let kpa = linearize(raw); // pretend this is an expensive polynomial correction
        *self.cache.borrow_mut() = Some((raw, kpa));
        kpa
    }
}
```

**Why this way:** exposing a read-only (`&self`) public API while updating
an internal cache is exactly the case `RefCell` exists for, single-
threaded — the moment this same cache needs touching from an interrupt
handler too, the fix is the `critical_section::Mutex<RefCell<_>>` pattern
above, not a thread-unsafe `RefCell` shared unguarded.

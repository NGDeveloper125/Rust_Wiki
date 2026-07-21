---
title: "Contain unsafety in small modules"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Encapsulation"]
related_syntax: [unsafe, mod]
see_also: ["Unsafe Rust", "Modules", "Visibility & privacy (pub and friends)"]
---

## Explanation

This idiom is about *where the boundary of trust lives* around unsafe
code, not about `unsafe` itself — see [Unsafe Rust](../memory-unsafe/unsafe-rust.md)
for the mechanism and what the keyword actually changes. The idea:
instead of scattering `unsafe` blocks across a codebase wherever a raw
pointer or FFI call happens to be convenient, gather everything that
depends on a given unsafe invariant into one small, deliberately scoped
[module](../modules-crates-visibility/modules.md), keep every field and
helper that participates in the invariant private to that module, and
expose only a safe API at the module's boundary. Every caller outside
the module then gets ordinary safe Rust back — the compiler re-enforces
all of its usual guarantees the moment code crosses back out — because
the module's author has already discharged the unsafe contract once, in
one reviewable place, instead of it being re-litigated at every call
site that happens to need it.

The size of the module matters as much as its existence. A module with
five unsafe operations spread across a thousand lines of unrelated safe
helper code is barely better than no containment at all — a reviewer
auditing "everywhere this invariant could be broken" still has to read
the whole file. A module of thirty focused lines wrapping one raw
pointer and nothing else is auditable in a single sitting, and — just as
importantly — small enough that every private field it touches can be
verified as reachable *only* through the module's own methods, since
[visibility](../modules-crates-visibility/visibility-and-privacy.md) is
what makes "only this module's code can violate this invariant" true in
the first place. Widening the module later — adding unrelated public
methods, letting other code obtain a raw pointer out of it — directly
widens the audit surface back out.

This is why the idiom pairs naturally with FFI wrappers and low-level
data structures: a foreign library's opaque handle, or a hand-rolled
allocator's raw pointer arithmetic, gets wrapped in a small module
exposing a safe struct, safe methods, and often a `Drop` implementation
that releases the resource — everything the unsafe code depends on
stays a private implementation detail the module alone is responsible
for. The same shape shows up constantly in embedded hardware-abstraction
crates: a peripheral's raw register access is unsafe, but the module
wraps it in a small, safe, typed API so application code above it never
writes `unsafe` at all.

## Basic usage example

```
mod counter {
    use std::cell::UnsafeCell;

    pub struct FastCounter { // <- the only public item; every unsafe detail stays inside this module
        value: UnsafeCell<u32>,
    }

    impl FastCounter {
        pub fn new() -> Self {
            Self { value: UnsafeCell::new(0) }
        }

        pub fn increment(&self) {
            unsafe {
                // SAFETY: called only through `&self` from single-threaded code in this example;
                // no other reference to `value` is live at the same time.
                *self.value.get() += 1;
            }
        }

        pub fn get(&self) -> u32 {
            unsafe { *self.value.get() }
        }
    }
}

let counter = counter::FastCounter::new();
counter.increment();
println!("{}", counter.get());
```

## Best practices & deeper information

### Scenario: Designing a public API

A small bump-allocator hands out slots from one pre-allocated buffer;
every pointer-arithmetic operation stays inside the module, and the
public API only ever returns safe references tied to the arena's own
lifetime.

```
mod arena {
    pub struct Arena {
        buffer: Vec<u8>,
        used: usize,
    }

    impl Arena {
        pub fn new(capacity: usize) -> Self {
            Self { buffer: vec![0; capacity], used: 0 }
        }

        pub fn alloc(&mut self, bytes: usize) -> &mut [u8] {
            assert!(self.used + bytes <= self.buffer.len(), "arena exhausted");
            let start = self.used;
            self.used += bytes;
            unsafe {
                // SAFETY: `start..start + bytes` was just checked to be within
                // `self.buffer`'s bounds, and no other slice into `buffer` is live.
                std::slice::from_raw_parts_mut(self.buffer.as_mut_ptr().add(start), bytes) // <- the module's one raw-pointer operation
            }
        }
    }
}

let mut arena = arena::Arena::new(64);
let slot = arena.alloc(8); // <- caller only ever sees a safe `&mut [u8]`, never a raw pointer
slot[0] = 42;
```

**Why this way:** the only place `Arena`'s bounds invariant could be
violated is inside this one module, which is exactly the small,
auditable "unsafe core with a safe API around it" shape the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
recommends — applied here to a bump allocator instead of a fixed
buffer.

### Scenario: Crossing an FFI boundary

Wrapping a C standard library function so the single `unsafe extern`
call and its safety justification live in one place, with the rest of
the crate only ever seeing a safe, callable type.

```
mod libm {
    unsafe extern "C" {
        fn abs(input: i32) -> i32;
    }

    pub struct SafeAbs; // <- the module's only public item

    impl SafeAbs {
        pub fn compute(&self, input: i32) -> i32 {
            unsafe {
                // SAFETY: `abs` from the C standard library is a pure function
                // with no preconditions on its i32 argument.
                abs(input) // <- the only place in the crate this raw FFI call is allowed to happen
            }
        }
    }
}

let calculator = libm::SafeAbs;
println!("{}", calculator.compute(-7));
```

**Why this way:** keeping the single point where the program vouches for
a foreign function's contract private to one module, rather than letting
`unsafe extern` calls appear anywhere a crate feels like reaching for
them, is exactly the containment the
[Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
assumes when it discusses trusting a foreign function's behavior.

## Explanation (Embedded)

This idiom isn't just applicable to embedded Rust — it's arguably the
organizing principle the whole embedded Rust ecosystem is layered around.
[Unsafe Rust](../memory-unsafe/unsafe-rust.md)'s and [Raw
pointers](../memory-unsafe/raw-pointers.md)'s embedded notes cover the
*mechanics* a contained module has to get right — volatile access,
interrupt-safety, the exact register semantics a datasheet specifies —
so this page won't repeat that ground. What's worth going deeper on here
is the *module-boundary design* itself: how embedded crates draw the
line, and a guarantee the boundary provides on top of hardware that a
purely software small-module boundary (a bump allocator, an FFI wrapper)
doesn't get for free.

The ecosystem's standard layering is three tiers, each one a small-module
boundary around the tier below it. A **peripheral access crate** (a
"PAC," usually generated from the chip vendor's SVD file by `svd2rust`)
is the innermost layer: it exposes one struct per peripheral, one method
per register, and every method is `unsafe` — it's a thin, mechanical
wrapper around raw addresses with essentially no judgment applied. A
**HAL crate** (`stm32f4xx-hal`, `nrf52840-hal`, and similar) wraps the
PAC: it is the module boundary this idiom is actually about — it takes
ownership of the PAC's raw register structs and exposes safe, typed
methods (`led.set_high()`, `i2c.write(addr, &bytes)`) built on the shared
`embedded-hal` traits, with the unsafe register pokes contained entirely
inside the HAL crate's own implementation. Everything above that —
driver crates, application firmware — depends only on the HAL's safe
trait methods and, ideally, never writes `unsafe` at all. The discipline
this page describes at the scale of one hand-written module is, in
embedded Rust, applied automatically by a whole layer of generated and
hand-written crates most firmware never needs to look inside.

The guarantee worth calling out that a software-only small module doesn't
automatically get: embedded aliasing is a *hardware* correctness
property, not just a Rust data-race property. Two independent `&mut`
handles to the same register block are just as unsound as two aliased
raw pointers in any other unsafe module, but the failure mode is a
control system with two independent writers to the same physical device
— a motor driver configured one way by task A and reconfigured
mid-operation by task B, neither aware the other exists. HAL crates
enforce the boundary at the *type* level, not just by convention:
`Peripherals::take()` returns `Option<Peripherals>` and can only ever
return `Some` once per program (a second call gets `None`, backed by a
hidden flag), so ownership itself — not just the module's privacy and the
author's discipline — statically prevents two independent driver
instances from ever both claiming the same register block. That's a
stronger boundary than "the unsafe fields are private and a reviewer
checked the invariant"; it's "the compiler refuses to hand out a second
handle at all."

## Basic usage example (Embedded)

```
mod led {
    const GPIOA_ODR: *mut u32 = 0x4001_0814 as *mut u32; // <- stays private to this module

    pub struct Led; // <- the only public item; every raw register detail stays inside `led`

    impl Led {
        pub fn set_high(&self) {
            unsafe {
                // SAFETY: see Raw pointers (Embedded) for the volatile-access rationale this
                // module relies on; contained here, this is the crate's only touch of GPIOA_ODR.
                let current = core::ptr::read_volatile(GPIOA_ODR);
                core::ptr::write_volatile(GPIOA_ODR, current | (1 << 5));
            }
        }
    }
}

let status_led = led::Led;
status_led.set_high(); // <- application code never sees a raw pointer or an address
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A board's LED and its user button both live on the same GPIO port, and
the module boundary needs to guarantee no two parts of the firmware can
ever independently claim the same underlying register block — not just
by convention, but so the compiler refuses a second claim outright.

```
struct GpioaRegisters; // stands in for a PAC-generated raw register struct

static mut TAKEN: bool = false;

pub struct Peripherals {
    pub gpioa: GpioaRegisters,
}

impl Peripherals {
    /// Returns the peripherals exactly once per program; a second call gets `None`.
    pub fn take() -> Option<Self> {
        unsafe {
            // SAFETY: firmware is single-threaded at startup, before interrupts are enabled,
            // so this check-and-set has no concurrent caller to race against.
            if TAKEN {
                return None;
            }
            TAKEN = true;
        }
        Some(Peripherals { gpioa: GpioaRegisters })
    }
}

let peripherals = Peripherals::take().expect("peripherals not yet taken");
let second_attempt = Peripherals::take();
assert!(second_attempt.is_none()); // <- the type system, not a reviewer, prevents a second owner of GPIOA
```

**Why this way:** privacy alone would stop *other modules'* code from
constructing a `GpioaRegisters` out of thin air, but it wouldn't stop the
same module from handing out the register block twice; the `take()`
pattern used throughout the `cortex-m`/chip-HAL ecosystem (see the [Rust
Embedded Book's peripherals
chapter](https://docs.rust-embedded.org/book/peripherals/singletons.html))
closes that gap by making "at most one owner" a runtime-enforced,
type-level fact, which is a strictly stronger containment guarantee than
a small module gets on a purely software invariant.

### Scenario: Crossing an FFI boundary

A vendor ships a proprietary radio SDK as a C library with dozens of
functions; rather than letting `unsafe extern "C"` calls spread across
the firmware wherever radio functionality is needed, one module owns
every call into the SDK and translates its error codes into a typed Rust
`Result`.

```
mod radio_sdk {
    unsafe extern "C" {
        fn radio_init() -> i32;
        fn radio_send(data: *const u8, len: u32) -> i32;
    }

    pub struct RadioError(pub i32);

    pub struct Radio; // <- the module's only public item

    impl Radio {
        pub fn init() -> Result<Self, RadioError> {
            let code = unsafe {
                // SAFETY: `radio_init` has no preconditions per the vendor SDK's header comment.
                radio_init() // <- the only place in the crate this call is allowed to happen
            };
            if code == 0 { Ok(Radio) } else { Err(RadioError(code)) }
        }

        pub fn send(&self, data: &[u8]) -> Result<(), RadioError> {
            let code = unsafe {
                // SAFETY: `data` is a valid Rust slice for its full length, which satisfies
                // `radio_send`'s documented pointer+length contract.
                radio_send(data.as_ptr(), data.len() as u32)
            };
            if code == 0 { Ok(()) } else { Err(RadioError(code)) }
        }
    }
}

let radio = radio_sdk::Radio::init().expect("radio init failed");
radio.send(b"ping").expect("send failed");
```

**Why this way:** every one of the vendor SDK's undocumented assumptions
about calling context lives behind this one module's two methods instead
of being re-litigated at every call site across the firmware that wants
to send a radio packet — the same "thin unsafe core, safe API" shape
[Unsafe Rust (Embedded)](../memory-unsafe/unsafe-rust.md)'s FFI scenario
describes, scaled up here to a whole vendor SDK with typed error
translation instead of one function.

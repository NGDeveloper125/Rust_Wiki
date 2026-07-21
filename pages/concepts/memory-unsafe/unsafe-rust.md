---
title: "Unsafe Rust"
area: "Memory & Unsafe"
embedded_support: full
groups: ["Memory & Unsafe", "Systems / Low-Level Programming", "Interfacing with C / Other Languages", "Unique to Rust", "Coming from C / C++"]
related_syntax: [unsafe, extern, "*"]
see_also: ["Raw pointers (*const T / *mut T)", "FFI (foreign function interface)", "Memory layout & repr", "The undefined-behavior boundary", "Smart pointers (Box<T>)", "Stack vs heap allocation"]
---

## Explanation

Unsafe Rust is the subset of the language where the compiler stops
checking a handful of specific guarantees and instead trusts the
programmer to uphold them by hand. It is not a different language, a
"turn off the borrow checker" switch, or a way to skip type checking —
ordinary Rust rules (types, lifetimes on bindings, most borrow rules)
still apply inside an `unsafe` block. What changes is narrow and
specific: only five extra operations become legal — dereferencing a raw
pointer, calling an `unsafe fn` or unsafe trait method, accessing or
mutating a mutable `static`, implementing an `unsafe trait`, and
accessing a union field.

Unsafe Rust exists because some operations are fundamentally impossible
for a compiler to verify at compile time, yet are still necessary:
talking to hardware registers, calling into a C library, building a data
structure whose safety invariant is more subtle than what the borrow
checker can express (like a doubly linked list or a custom `Vec`), or
implementing the very primitives — `Vec`, `Rc`, `Mutex` — that safe Rust
is built on top of. Someone has to write the unsafe code at the bottom of
the stack; Rust's design lets that code exist as an explicit, clearly
marked, opt-in exception rather than as an ambient possibility anywhere
in the language, which is what "safe by default, unsafe by exception"
means in practice.

The mental model worth keeping is a contract, not a compiler bypass:
every unsafe operation has an invariant a human must guarantee, and
`unsafe` is the keyword that says "I checked, trust me" at that one spot.
Violating the invariant doesn't produce a compiler error or even
necessarily a crash — it produces [undefined behavior](the-undefined-behavior-boundary.md),
which can manifest as anything from a wrong answer to a security hole,
often far away from the unsafe block that actually caused it. This is
why the idiomatic pattern is to keep unsafe code in small, heavily
documented modules and expose only a safe API to the rest of the program
— callers of the safe wrapper get all of Rust's usual guarantees back,
because the module's author has already discharged the unsafe contract
once, carefully, in one place.

Unsafe Rust is also the foundation the other pages in this group build
on: [raw pointers](raw-pointers.md) are the values unsafe code
dereferences, [FFI](ffi.md) is the single most common reason to write
`unsafe` at all, and [`repr`](memory-layout-and-repr.md) controls the
memory layout that unsafe code frequently has to reason about explicitly.

## Basic usage example

```
let mut num = 5;
let r = &mut num as *mut i32; // <- creating a raw pointer is safe...

unsafe {
    // SAFETY: `r` was just derived from a valid, live `&mut i32`, so it
    // points to a properly initialized, aligned i32 for the duration of
    // this block.
    *r += 1; // <- ...but dereferencing one requires `unsafe`
}
println!("{num}");
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

Calling a C library's logging function is the textbook reason to write
`unsafe` at all: the compiler has no way to verify what a foreign
function does with its arguments.

```
// Rustonomicon-style FFI example (std-only, no crate needed).
unsafe extern "C" {
    fn abs(input: i32) -> i32; // <- foreign function; the compiler cannot check its body
}

fn absolute_value(x: i32) -> i32 {
    unsafe {
        // SAFETY: `abs` from the C standard library is a pure function
        // with no preconditions on its i32 argument.
        abs(x) // <- calling into C requires unsafe: the compiler can't verify the FFI contract
    }
}

fn main() {
    println!("{}", absolute_value(-4));
}
```

**Why this way:** the compiler cannot check a foreign function's body, its
argument validity requirements, or its side effects, so `unsafe` marks the
exact point where the programmer is vouching for the call instead of the
compiler — the [Rustonomicon's FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html)
frames every `extern` call this way.

### Scenario: Designing a public API

The idiom "contain unsafety in small modules" means wrapping an unsafe
core in a safe, misuse-resistant API, so the rest of the codebase never
needs to write `unsafe` itself or reason about the invariant directly.

```
pub struct FixedBuffer {
    data: Vec<u8>,
    len: usize,
}

impl FixedBuffer {
    pub fn new(capacity: usize) -> Self {
        Self { data: vec![0; capacity], len: 0 }
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.len == self.data.len() {
            return false; // <- safe wrapper enforces the invariant unsafe code below relies on
        }
        unsafe {
            // SAFETY: `self.len < self.data.len()` was just checked above,
            // so this index is in bounds and the write is aligned.
            *self.data.get_unchecked_mut(self.len) = byte;
        }
        self.len += 1;
        true
    }
}
```

**Why this way:** `get_unchecked_mut` skips the bounds check that
`Vec::push` normally performs, so the module keeps its own invariant
(`len <= data.len()`) enforced entirely by the safe `push` method — the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
recommends exactly this shape: a small unsafe core with a safe API around
it, so a caller can never observe or trigger the broken invariant.

## Explanation (Embedded)

The [`unsafe` keyword page](../../syntax/keywords/unsafe.md) already
covers *why* embedded code reaches for `unsafe` constantly — register
access and vendor FFI calls are two of its five gated operations doing
real, unavoidable work. What belongs here, at the concept level, is the
design discipline that keeps that unavoidable `unsafe` from spreading
through an entire firmware codebase: contain it to a thin, deliberately
narrow hardware-abstraction-layer (HAL) module, and let every line of
application code above that module stay entirely safe.

This containment matters more in embedded Rust than almost anywhere
else, because the invariant an embedded `unsafe` block relies on is
often *physical*, not just logical: a raw pointer dereference is sound
only if the address is really mapped to the peripheral the code thinks
it is, a `static mut` is sound only if the interrupt that can also touch
it is masked or the access is genuinely atomic, and a vendor FFI call is
sound only if the C library's undocumented assumptions about calling
context are actually met. None of that is checkable by the compiler, and
re-deriving it at every call site multiplies the chance of getting it
wrong. The `embedded-hal` ecosystem's whole design point is this
containment done well: a chip-specific HAL crate owns every raw register
dereference and every `unsafe extern "C"` call, and exposes a small set
of safe, trait-based methods (`set_high()`, `read()`, `write(&bytes)`)
that the rest of the firmware — sensor drivers, application logic, RTOS
tasks — builds on without ever writing `unsafe` itself. A firmware
project where `unsafe` is scattered through business logic instead of
concentrated in one audited layer has usually skipped this discipline,
not encountered some embedded-specific necessity for it.

## Basic usage example (Embedded)

```
const TIMER_CTRL: *mut u32 = 0x4000_0000 as *mut u32;

/// Starts the hardware timer. Contained to this one function; callers
/// elsewhere in the firmware never touch TIMER_CTRL directly.
pub fn start_timer() {
    unsafe {
        // SAFETY: TIMER_CTRL is the timer peripheral's real control
        // register on this chip, and setting bit 0 is its documented
        // "enable" operation.
        core::ptr::write_volatile(TIMER_CTRL, 0x1);
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A temperature sensor driver's entire unsafe surface is one raw pointer
read; everything a caller sees is a safe function returning a typed
value.

```
mod sys {
    pub const SENSOR_DATA: *const u32 = 0x4003_4000 as *const u32; // <- raw address stays private to this module
}

pub struct TemperatureSensor;

impl TemperatureSensor {
    pub fn read_celsius(&self) -> f32 {
        let raw = unsafe {
            // SAFETY: SENSOR_DATA is always mapped on this chip; a plain
            // volatile read has no other preconditions.
            core::ptr::read_volatile(sys::SENSOR_DATA) // <- the only unsafe operation in the whole driver
        };
        (raw as f32) * 0.0625 // datasheet's raw-to-Celsius conversion
    }
}
```

**Why this way:** `sys::SENSOR_DATA` never leaves the module, so
`TemperatureSensor::read_celsius` is the *only* place in the entire crate
an auditor needs to check against the datasheet — the same "thin unsafe
core, safe API" idiom the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
describes for any unsafe module, applied to a register instead of a data
structure.

### Scenario: Sharing state across threads

A tick counter incremented by a timer interrupt and read from the main
loop is embedded Rust's closest analogue to sharing state across
threads — the interrupt handler is a genuinely concurrent execution
context, just one triggered by hardware instead of `thread::spawn`.

```
static mut TICK_COUNT: u32 = 0;

// Registered as the timer interrupt handler; runs at any point, preempting main.
#[allow(non_snake_case)]
fn TIM2() {
    unsafe {
        // SAFETY: interrupts of the same priority don't nest on this
        // chip, so no other write to TICK_COUNT can be in progress here.
        let ptr = &raw mut TICK_COUNT;
        *ptr = ptr.read_volatile().wrapping_add(1);
    }
}

pub fn ticks() -> u32 {
    cortex_m::interrupt::free(|_| unsafe {
        // SAFETY: interrupts are masked for the duration of this closure,
        // so TIM2 cannot preempt this read.
        core::ptr::read_volatile(&raw const TICK_COUNT)
    })
}
```

**Why this way:** without masking interrupts around the main-loop read,
`ticks()` could observe `TICK_COUNT` mid-update if `TIM2` fires between a
load and a store elsewhere in the program — `cortex_m::interrupt::free`
is the embedded equivalent of a mutex critical section, and `&raw
mut`/`&raw const` (see [`&raw const`/`&raw
mut`](../../syntax/operators/raw-borrow.md)) form the pointer without
ever asserting the `static mut` is exclusively borrowed, which a plain
`&mut TICK_COUNT` cannot honestly claim from inside an interrupt
handler.

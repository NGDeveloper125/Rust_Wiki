---
title: "The borrow checker"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Unique to Rust", "Coming from Python / JavaScript", "Coming from Java / C#", "Coming from C / C++"]
related_syntax: ["&", mut]
see_also: ["Ownership", "Borrowing (shared references)", "Mutable borrowing", "Lifetimes"]
---

## Explanation

The borrow checker is the part of the compiler that statically verifies
every ownership and borrowing rule holds for every possible execution
path, before the program ever runs: no value is used after it's moved, no
reference outlives the data it points to, and no mutable reference
coexists with any other reference to the same data.

This is what most people mean when they talk about "fighting the borrow
checker" as a newcomer — it rejects programs that would be memory-unsafe
(dangling pointers, data races, use-after-free) at compile time, with no
runtime cost and no possibility of the bug reaching production, in
exchange for sometimes requiring the code to be restructured in ways that
feel unfamiliar coming from a language without this check.

The mental model that helps most: the borrow checker isn't an arbitrary
obstacle, it's checking a real property your program needs to hold
regardless of language — other languages either enforce a version of it
at runtime (garbage collection sidesteps use-after-free by never freeing
early; a mutex enforces exclusivity at runtime with a lock) or don't
enforce it at all (raw pointers in C, where violating it is undefined
behavior). Rust's distinguishing choice is doing the check entirely at
compile time, for zero runtime cost, which is only possible because the
rules are conservative — some genuinely safe programs are rejected simply
because the checker can't prove they're safe, which is why escape hatches
like [interior mutability](interior-mutability.md), reference counting,
and (in the rare, unavoidable case) `unsafe` exist.

## Basic usage example

```
let mut v = vec![1, 2, 3];
let first = &v[0];       // <- shared borrow of v starts here
v.push(4);                // borrow checker error: can't mutate v...
println!("{first}");      // ...while `first` still borrows it
```

**Restriction:** this is a compile-time rejection, not a runtime panic —
the fix is to shorten `first`'s scope (or restructure the code) so it no
longer overlaps with the mutable use.

## Best practices & deeper information

### Scenario: Sharing data with multiple references

Reading a shared config in several places goes smoothly as long as each
borrow's scope is kept as small as the code that actually needs it,
rather than held open for the rest of a function "just in case."

```
struct Config {
    max_retries: u32,
    timeout_ms: u32,
}

fn summarize(cfg: &Config) -> String {
    format!("retries={} timeout={}", cfg.max_retries, cfg.timeout_ms)
}

let cfg = Config { max_retries: 3, timeout_ms: 500 };

{
    let view = &cfg; // <- borrow scoped tightly to this block
    println!("{}", summarize(view)); // <- last use: the borrow ends here (NLL)
} // well before any later mutation of cfg would need to happen
```

**Why this way:** letting a shared borrow end at its last use, rather
than at the end of its enclosing scope, is what keeps later
mutations of the same value from ever colliding with it — the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
covers this scoping behavior as the key to avoiding borrow-checker
friction, not a workaround for it.

### Scenario: Mutating through a reference

Updating one field of a struct while still needing to read another works
in a single scope because the borrow checker tracks borrows of individual
fields separately — a mutable borrow of one field can coexist with a
shared borrow of a different one.

```
struct Sensor {
    readings: Vec<f64>,
    status: String,
}

let mut sensor = Sensor { readings: Vec::new(), status: "ok".into() };

let readings = &mut sensor.readings; // <- mutable borrow of ONE field
let status = &sensor.status;         // <- simultaneous shared borrow of a DIFFERENT field
readings.push(21.5);
println!("{status}");
```

**Why this way:** the borrow checker tracks field-level (disjoint)
borrows within a function body, so structuring mutation around individual
fields rather than whole-struct `&mut self` methods avoids artificial
conflicts; the
[Rustonomicon](https://doc.rust-lang.org/nomicon/borrow-splitting.html)
covers splitting borrows across separate fields as the standard way to
keep the checker satisfied without restructuring the data itself.

## Explanation (Embedded)

The borrow checker's proof happens entirely at compile time and leaves no
trace in the compiled output, which is exactly the property that makes it
uniquely valuable in code shared between `fn main`'s loop and one or more
`#[interrupt]` handlers. An interrupt can preempt `main` at essentially any
instruction boundary, completely independent of program logic — from the
compiler's point of view, an interrupt handler is a second concurrent
context that can run "at the same time" as `main`, with none of the
sequencing guarantees an ordinary function call provides. In C, sharing a
global between a handler and the main loop is just sharing a global:
nothing stops the same multi-byte struct from being half-written by the
handler and half-read by main, and the resulting corruption is a runtime
heisenbug that only reproduces under the right timing — found, if you're
lucky, in testing, and if not, months later in the field. Rust's borrow
checker turns that entire bug class into a compile error: safe Rust has no
way to alias a `&mut` across the main-loop/interrupt boundary without
going through something the checker can verify, so a `static` item touched
from both contexts must be wrapped in a synchronization primitive
(typically `critical_section::Mutex<Cell<T>>`/`Mutex<RefCell<T>>`, see
[Interior mutability](interior-mutability.md)) before either side can
safely reach it — and the compiler simply refuses to build anything that
tries to skip that step. None of this costs a single cycle at runtime: the
check runs once, on the host machine, during compilation, and the firmware
that ships contains no borrow-tracking code whatsoever. It's the identical
static analysis described above, just applied to a domain — interrupt-
driven, no-OS firmware — where the class of bug it prevents is unusually
common and unusually expensive to debug once it's running on real
hardware.

## Basic usage example (Embedded)

```
struct SensorConfig { gain: u8 }

let mut config = SensorConfig { gain: 4 };
let cfg_ref = &config;     // <- shared borrow of config starts here
config.gain = 8;            // borrow checker error: can't mutate while borrowed
println!("{}", cfg_ref.gain);
```

## Best practices & deeper information (Embedded)

### Scenario: Sharing state across threads

A timer interrupt updates the latest sample while the main loop reads it;
the borrow checker won't let either side reach the shared value except
through a synchronization wrapper it can verify.

```
use core::cell::Cell;
use critical_section::Mutex;

// AVOID: a bare mutable static — nothing stops `main` and the interrupt
// handler from touching it at the same instant, and the aliasing bug is
// only caught (if ever) as a corrupted reading discovered on hardware
// static mut LATEST_SAMPLE: u16 = 0;

// PREFER: the borrow checker enforces exclusive access at compile time
static LATEST_SAMPLE: Mutex<Cell<u16>> = Mutex::new(Cell::new(0));

#[interrupt]
fn ADC1() {
    critical_section::with(|cs| {
        LATEST_SAMPLE.borrow(cs).set(read_adc_register()); // <- only reachable inside a checked critical section
    });
}

fn main_loop() -> ! {
    loop {
        let sample = critical_section::with(|cs| LATEST_SAMPLE.borrow(cs).get());
        process(sample);
    }
}
```

**Why this way:** wrapping the shared sample in `critical-section`'s
`Mutex<Cell<_>>` is what lets the borrow checker prove `main` and the
interrupt handler never alias a `&mut` to the same value — removing the
wrapper doesn't just risk a bug, it fails to compile, since Rust's 2024
edition denies-by-default creating a reference to a mutable `static` via
[`static_mut_refs`](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html)
specifically because that pattern can't be checked; C's shared-global
idiom offers no equivalent compile-time backstop.

### Scenario: Mutating through a reference

A HAL's `split()` call turns one peripheral register block into disjoint,
individually-owned fields specifically so the borrow checker can reason
about each pin's borrows independently of the others.

```
let gpiob = dp.GPIOB.split(); // <- returns disjoint per-pin fields, not one shared handle

let led = &mut gpiob.pb0;   // <- exclusive borrow of ONE pin
let button = &gpiob.pb1;    // <- simultaneous shared borrow of a DIFFERENT pin
led.set_high();
if button.is_high() {
    // ...
}
```

**Why this way:** the borrow checker tracks field-level (disjoint) borrows
within a function body, so `embedded-hal`-style crates deliberately split
a peripheral into one field per pin/function — the same field-splitting
mechanism the
[Rustonomicon](https://doc.rust-lang.org/nomicon/borrow-splitting.html)
documents for ordinary structs, applied here to real driver design so
unrelated pins never contend over one shared handle.

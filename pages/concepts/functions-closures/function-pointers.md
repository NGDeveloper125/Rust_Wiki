---
title: "Function pointers (fn types)"
area: "Functions & Closures"
embedded_support: full
groups: ["Functions & Closures"]
related_syntax: [fn, "->", "( )"]
see_also: ["Closures & capturing", "Fn / FnMut / FnOnce", "Higher-order functions"]
---

## Explanation

A function pointer is a plain value type like `fn(i32) -> i32`: it holds
only the address of a function, nothing else. Every named
[function](functions.md) has a concrete type of its own, but that type
can always be coerced to the matching `fn(...) -> ...` pointer type,
which is what you actually write down when a function's type needs to be
named — as a field type, a parameter type, or an array element type.

This is distinct from a [closure's](closures-and-capturing.md) type: a
closure that captures anything is backed by a compiler-generated
anonymous struct sized to hold exactly what it captured, so no single
named type describes every closure with a given signature. A function
pointer, in contrast, is always exactly one pointer wide, `Copy`, and
`'static`, regardless of which matching function it currently holds — the
tradeoff is that it can never carry an environment, since there's nowhere
in a bare pointer for captured state to live.

Because a function pointer captures nothing, it automatically implements
all three of [`Fn`, `FnMut`, and `FnOnce`](fn-fnmut-fnonce.md) — so any
place that accepts a closure by one of those bounds also accepts a plain
function name directly, with no extra syntax. This is also why a
[higher-order function](higher-order-functions.md) can choose to take a
bare `fn(...) -> ...` parameter instead of a generic closure bound when
it will never need to accept a capturing closure: doing so keeps the
function itself non-generic, at the cost of ruling out capturing closures
as arguments.

Reaching for a plain function pointer makes the most sense when every
value that will ever be stored is a non-capturing function anyway — a
dispatch table of named operations, a callback slot that's set once and
never needs surrounding context, or an FFI-style callback signature
shared with C.

## Basic usage example

```
fn square(n: i32) -> i32 { n * n }

let op: fn(i32) -> i32 = square; // <- `op`'s type is a function pointer, not a closure
println!("{}", op(6));
```

**Restriction:** only a function (or non-capturing closure) can be
assigned to a `fn(...) -> ...` binding — a closure that captures anything
fails to coerce, since there's no environment slot in a bare pointer to
hold it.

## Best practices & deeper information

### Scenario: Designing a public API

A logging sink's formatter never needs to capture request-specific
state, so its field is typed as a plain function pointer instead of a
boxed closure — smaller, `Copy`, and simpler to construct.

```
struct Logger {
    formatter: fn(level: &str, message: &str) -> String,
    // <- fn pointer: no capture ever needed, fixed-size, and Copy
}

fn plain_formatter(level: &str, message: &str) -> String {
    format!("[{level}] {message}")
}

impl Logger {
    fn new() -> Self {
        Logger { formatter: plain_formatter }
    }

    fn log(&self, level: &str, message: &str) {
        println!("{}", (self.formatter)(level, message));
    }
}
```

**Why this way:** a `Box<dyn Fn(...) -> ...>` field would also work here,
but it costs a heap allocation and a dynamic dispatch for a value that
never captures anything — the
[API Guidelines' flexibility guidance](https://rust-lang.github.io/api-guidelines/flexibility.html)
favors the simplest type that satisfies the actual requirement.

### Scenario: Writing generic code

A small calculator dispatches by operation name using a lookup table of
named operations, all sharing one concrete `fn(f64, f64) -> f64` type —
no generic parameter is needed since every entry has the same shape.

```
const OPERATIONS: &[(&str, fn(f64, f64) -> f64)] = &[
    // <- an array of fn pointers: every closure below captures nothing, so all coerce to one type
    ("add", |a, b| a + b),
    ("sub", |a, b| a - b),
    ("mul", |a, b| a * b),
];

fn run_operation(name: &str, a: f64, b: f64) -> Option<f64> {
    OPERATIONS
        .iter()
        .find(|(op_name, _)| *op_name == name)
        .map(|(_, op)| op(a, b))
}
```

**Why this way:** a non-capturing closure coerces to a function pointer
automatically wherever a `fn` type is expected, which the
[Rust Reference](https://doc.rust-lang.org/reference/type-coercions.html)
lists as one of the standard coercions — letting a table like this mix
closure literals and named functions under one concrete element type.

### Scenario: Working with collections

Applying one of several unit-conversion functions across a batch of
sensor readings is a single, non-generic helper: the conversion is a
parameter typed as a function pointer, not a generic closure bound.

```
fn to_fahrenheit(c: f64) -> f64 { c * 9.0 / 5.0 + 32.0 }
fn to_kelvin(c: f64) -> f64 { c + 273.15 }

fn convert_all(readings: &[f64], convert: fn(f64) -> f64) -> Vec<f64> {
    // <- `convert` is a plain fn pointer parameter, not a generic `F: Fn(f64) -> f64`
    readings.iter().map(|&c| convert(c)).collect()
}

let fahrenheit_readings = convert_all(&[0.0, 20.0, 100.0], to_fahrenheit);
```

**Why this way:** because a `fn` pointer parameter isn't generic,
`convert_all` compiles to a single function regardless of which
conversion function is passed in, unlike a generic `F: Fn(f64) -> f64`
parameter, which the
[standard library docs](https://doc.rust-lang.org/std/primitive.fn.html)
note gets monomorphized separately per distinct closure or function
passed in.

## Explanation (Embedded)

A hardware interrupt/exception vector table is, at the machine level,
nothing more than a fixed-size array of addresses the CPU jumps to
directly on reset or interrupt — which is exactly what a Rust `fn()`
pointer is: an address with no captured environment attached. This makes
`fn` pointers (never closures) the natural, and often mandatory, type
behind a `#[link_section = ".vector_table"]` static array of handlers:
the linker script expects a table of fixed-width words at a fixed
address, and only a non-capturing function pointer has the fixed, known
size (and `'static`, `Copy` nature) that shape requires. A capturing
closure couldn't go in that table at all — there's no environment slot
in a bare machine word for it to live in.

This is the same property that makes `fn` pointers useful in hosted
dispatch tables (see this page's classic Best practices), just with the
stakes raised: on embedded targets the array isn't a convenience, it's
read directly by hardware, so its element type is constrained by the
CPU's vector-table ABI rather than by API taste.

## Basic usage example (Embedded)

```
#[link_section = ".vector_table.reset"]
#[used]
static RESET_HANDLER: fn() -> ! = reset; // <- `fn() -> !`: fixed-size, no captured environment, placed at a fixed address

fn reset() -> ! {
    loop {}
}
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

A microcontroller's interrupt vector table is a static array read
directly by hardware, so every entry must be a plain, fixed-size `fn()`
pointer — never a closure, since a closure's size and layout can vary and
aren't part of the vector-table ABI the silicon expects.

```
#[link_section = ".vector_table.exceptions"]
#[used]
static EXCEPTIONS: [fn(); 2] = [
    // <- an array of bare `fn()` pointers: exactly what the hardware vector table's layout requires
    nmi_handler,
    hard_fault_handler,
];

fn nmi_handler() {
    loop {}
}

fn hard_fault_handler() {
    loop {}
}
```

**Why this way:** the vector table's address and layout are dictated by
the CPU vendor's reference manual, not by Rust — only a non-capturing
`fn()` pointer has the fixed word size and absence of an environment that
a linker-placed table of raw addresses can hold; the
[embedded Rust book's exception-handling chapter](https://doc.rust-lang.org/stable/embedded-book/start/exceptions.html)
builds vector tables from exactly this shape.

### Scenario: Designing a public API

A HAL's interrupt-registration slot for a peripheral that's set up once
at boot and never needs surrounding context is typed as a plain `fn()`
pointer field, keeping the peripheral driver itself free of any generic
parameter or heap allocation.

```
struct TimerConfig {
    on_tick: fn(), // <- fn pointer: fixed ABI, no capture, matches what the timer peripheral's ISR table expects
}

fn default_tick_handler() {
    // increment a tick counter
}

fn configure_timer() -> TimerConfig {
    TimerConfig { on_tick: default_tick_handler }
}
```

**Why this way:** a non-capturing handler set once at startup has nothing
to gain from a generic `Fn` bound or a boxed trait object — a bare
`fn()` field is the smallest, `Copy`, allocation-free type that satisfies
the requirement, and it composes directly with the vector-table pattern
above if the handler ever needs to be installed into hardware directly.

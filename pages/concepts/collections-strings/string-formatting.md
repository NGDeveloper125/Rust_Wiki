---
title: "String formatting (Display, Debug, format!)"
area: "Collections & Strings"
embedded_support: partial
groups: ["Collections & Strings", "String Handling"]
related_syntax: ["{ }", "format!", "!"]
see_also: ["String vs &str", "Vec<T>"]
---

## Explanation

Turning a value into text in Rust runs through the `core::fmt` machinery
that backs `println!`, `format!`, and every other formatting macro.
`Display` is the trait for a value's user-facing textual form — plugged
in via the `{}` placeholder — and it must be implemented by hand,
because Rust has no default opinion about how a custom type should look
to a user. `Debug` is the developer-facing form — `{:?}` — meant for
logs, error messages, and `assert!` output rather than end users, and it
can be derived automatically with `#[derive(Debug)]` for almost any type
made of already-`Debug` fields.

`format!` uses the exact same placeholder syntax as `println!`, but
instead of writing to standard output it builds and returns an owned
`String` — it's the go-to way to assemble text from a mix of literal
and interpolated parts without manual `String::push_str` calls. Both
macros parse the same mini-language inside `{}`: a bare `{}` picks up
the next argument's `Display` impl, `{:?}` picks up `Debug`, `{:#?}`
asks for `Debug`'s "pretty," multi-line form, and named/positional
arguments (`{name}`, `{0}`) select a specific value instead of the next
one in sequence.

Implementing `Display` is also what makes a type usable as a
`std::error::Error` — the trait requires `Display` for its user-facing
message, on top of `Debug`, which is why custom error types almost
always implement or derive both. Deriving `Debug` costs nothing and is
close to mandatory for any public type, since it's what shows up when
that value appears in a panic message, an `unwrap()` on a `Result`
containing it, or a debug log line.

Under the hood, both traits are just a method that writes into a
`&mut fmt::Formatter` — the same `Write` trait that backs writing
formatted text into any buffer, not only building a `String`, which is
what makes `core::fmt` usable even where heap allocation isn't
available (see Embedded Rust Notes below).

## Basic usage example

```
use std::fmt;

struct Order { id: u64 }

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Order #{}", self.id) // <- {} picks up self.id's own Display impl
    }
}

let order = Order { id: 42 };
let summary = format!("{order}"); // <- format! builds an owned String using the Display impl above
println!("{summary}"); // Order #42
```

**Restriction:** a type has no `{}` form until `Display` is implemented
for it by hand — unlike `Debug`, it can never be derived, since there's
no way for the compiler to guess what a "user-facing" rendering should
look like.

## Best practices & deeper information

### Scenario: Working with text

`Display` and `Debug` serve different audiences on the same type: derive
`Debug` for free developer-facing output, and hand-write `Display` only
for the subset of types that also need a clean, user-facing rendering.

```
use std::fmt;

#[derive(Debug)] // <- free, developer-facing form: {:?} shows every field
struct Temperature { celsius: f64 }

impl fmt::Display for Temperature { // <- hand-written, user-facing form: {} shows just the reading
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

let reading = Temperature { celsius: 21.456 };
println!("{reading}");   // 21.5°C          (Display)
println!("{reading:?}"); // Temperature { celsius: 21.456 } (Debug)
```

**Why this way:** the
[std docs](https://doc.rust-lang.org/std/fmt/index.html#formatting-traits)
describe `Display` as for "output intended for end users" and `Debug` as
for "programmer-facing output" — deriving one and hand-writing the other
keeps that distinction honest instead of overloading a single impl for
both audiences.

### Scenario: Handling and propagating errors

`std::error::Error` requires `Display`, so a custom error type's
`Display` impl doubles as the message shown wherever the error surfaces
— a `?`-propagated failure, a top-level `main` returning `Result`, or a
log line.

```
use std::fmt;

#[derive(Debug)]
enum OrderError {
    NotFound(u64),
    OutOfStock { sku: String },
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderError::NotFound(id) => write!(f, "order {id} not found"), // <- becomes the error's user-facing message
            OrderError::OutOfStock { sku } => write!(f, "sku {sku} is out of stock"),
        }
    }
}

impl std::error::Error for OrderError {} // <- requires Display (above) and Debug (derived)
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
and the `std::error::Error` trait itself require a `Display` impl
precisely so that propagating an error with `?` up to `main` or a log
call has a ready-made, readable message rather than only the derived
`Debug` dump.

### Scenario: Documenting an API

Deriving `Debug` on every public type means panics, `unwrap()`s, and log
statements that include the value all produce a readable dump for free,
which is invaluable when diagnosing a report from the field.

```
#[derive(Debug)] // <- shows up in panic messages and log lines with zero extra code
struct Shipment {
    order_id: u64,
    carrier: String,
}

fn dispatch(shipment: &Shipment) {
    println!("dispatching {shipment:?}"); // <- {:?} needs only the derive above
}

dispatch(&Shipment { order_id: 42, carrier: "Northwind".into() });
```

**Why this way:** the API Guidelines'
[C-DEBUG](https://rust-lang.github.io/api-guidelines/interoperability.html#c-debug)
item recommends every public type implement `Debug`, since library
consumers rely on it turning up automatically in their own logging and
panic output without having to write a formatter themselves.

## Explanation (Embedded)

The split this page draws between the `core::fmt` machinery and the
allocating macros built on top of it is exactly the line embedded support
falls on. `Display`, `Debug`, and `write!` all work in `#![no_std]` with
no allocator at all, because they only ever write through a
`Formatter`/`core::fmt::Write` sink the caller already owns — a custom
`Display` impl written the way this page's Basic usage example shows
compiles unchanged under `#![no_std]`. `format!` and `.to_string()`,
though, both *return* a freshly allocated `String`, so they need `alloc`
plus a configured `#[global_allocator]` the same way `Vec`/`String`
themselves do (see [String vs &str](string-vs-str.md)).

The realistic no-heap substitute isn't a different macro so much as a
different destination: instead of `format!("...")` producing an owned
`String`, `write!`/`writeln!` the same format string into a
`heapless::String<N>` or directly into a peripheral (a UART) that
implements `core::fmt::Write` — see
[`write!` / `writeln!`](../../syntax/macros/write-macros.md) for how that
destination-trait split works. This page's angle is which macro a caller
reaches for, not the mechanics of the trait: reach for `write!` into an
owned, bounded buffer or a peripheral wherever hosted code would reach
for `format!`.

For targets tight enough on flash that even `core::fmt`'s format-string
parsing and `Arguments` machinery are too much code size, the `ufmt`
crate is a real, narrower alternative: it defines its own leaner
`uDisplay`/`uDebug` traits and a `uwrite!`/`uwriteln!` macro pair with a
smaller compiled footprint than `core::fmt`, at the cost of supporting
fewer format specifiers. It's a deliberate trade of `core::fmt`'s
flexibility for less code, not a drop-in replacement for it.

## Basic usage example (Embedded)

```
use core::fmt::Write as _;
use heapless::String;

struct Order { id: u64 }

impl core::fmt::Display for Order {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Order #{}", self.id) // <- same Display impl shape as hosted Rust, no heap involved
    }
}

let order = Order { id: 42 };
let mut summary: String<32> = String::new();
write!(summary, "{order}").unwrap(); // <- writes into a fixed-capacity buffer instead of format!'s owned String
```

## Best practices & deeper information (Embedded)

### Scenario: Working with text

Rendering a sensor reading into a log line needs `format!`'s shape —
literal text interleaved with interpolated values — but not an
allocation; `write!` into a `heapless::String` is the substitute that
keeps the same `{}`/`{:.1}` grammar.

```
use core::fmt::Write as _;
use heapless::String;

fn render_reading(id: u32, celsius: f32) -> String<32> {
    // AVOID: format!("sensor {id}: {celsius:.1}C") -- needs alloc + a configured #[global_allocator]
    let mut line: String<32> = String::new();
    write!(line, "sensor {id}: {celsius:.1}C").unwrap(); // PREFER: writes into a fixed 32-byte buffer, no heap
    line
}
```

**Why this way:** `write!` and `format!` share identical formatting
syntax, so switching destinations costs nothing in readability — only
the buffer changes from an owned, unbounded `String` to a bounded
`heapless::String<N>`, per the same `core::fmt::Write` mechanics
[`write!` / `writeln!`](../../syntax/macros/write-macros.md) documents in
full.

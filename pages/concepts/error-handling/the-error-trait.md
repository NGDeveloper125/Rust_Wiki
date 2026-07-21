---
title: "The Error trait"
area: "Error Handling"
embedded_support: full
groups: ["Error Handling", "Handling Errors & Failure", "Error Propagation"]
related_syntax: ["?", dyn]
see_also: ["Custom error types", "Result<T, E>", "The ? operator (concept angle)", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

`std::error::Error` (now `core::error::Error`, stabilized there since
Rust 1.81) is the standard interface an error type is expected to
implement. Its requirements are small: a type must already implement
`Display` (a human-readable message) and `Debug` (a developer-facing
representation), and the trait itself adds exactly one method with a
default implementation — `source()`, returning
`Option<&(dyn Error + 'static)>`, the underlying cause this error wrapped,
if any. That's the entire contract; the trait carries no data of its
own.

It exists so that generic, error-agnostic code can exist at all. Without
a shared trait, a top-level `main`, a logging layer, or an application's
error-reporting boundary would need to know the concrete error type of
every fallible call it might ever see — completely impractical once a
program depends on more than a handful of crates. Implementing `Error`
is what lets an error be boxed into `Box<dyn Error>`, logged uniformly,
and chased back through `source()` to its root cause, regardless of
which crate defined it.

The mental model is a chain, not a single value: many failures are really
"this failed, because that failed, because the thing under *that*
failed." `source()` is how a custom error type exposes the next link
down without collapsing the whole chain into one string up front — a
caller (or a reporting tool) can walk `source()` repeatedly until it
returns `None`, printing or inspecting each cause in turn.

[Custom error types](custom-error-types.md) covers designing an error
enum's variants and data; this page is about the trait those types
implement to interoperate with the rest of the ecosystem. `thiserror`'s
derive generates an `Error` impl (and often `From` conversions) from
attributes, but it's still worth seeing the manual version at least
once — the examples below implement `Error` by hand, including a manual
`source()` and a manual `From` impl, so the derive's shorthand has a
concrete mechanism behind it rather than being magic.

A function boundary that doesn't want to commit to one concrete error
type at all can return `Result<T, Box<dyn Error>>` (or
`Box<dyn Error + Send + Sync>` when the box must cross threads), since
any type implementing `Error` can be boxed into it, and
[`?`](the-question-mark-operator.md) converts into that box
automatically — the same mechanism `anyhow::Error` builds a more
ergonomic wrapper around for application code.

## Basic usage example

```
use std::fmt;

#[derive(Debug)]
struct EmptyQueueError;

impl fmt::Display for EmptyQueueError { // <- Error requires Display for a human-readable message
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cannot dequeue: the queue is empty")
    }
}

impl std::error::Error for EmptyQueueError {} // <- the trait itself: no methods required beyond its supertraits here
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

A database layer wraps a lower-level I/O failure; implementing `Error`
manually (no `thiserror`) and overriding `source()` keeps the original
cause discoverable instead of losing it inside a formatted string.

```
use std::fmt;
use std::error::Error;

#[derive(Debug)]
struct QueryError {
    query: String,
    cause: std::io::Error,
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "query failed: {}", self.query)
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause) // <- exposes the underlying io::Error as the chained cause
    }
}

fn log_chain(err: &dyn Error) {
    eprintln!("{err}");
    let mut cause = err.source();
    while let Some(inner) = cause {
        eprintln!("caused by: {inner}");
        cause = inner.source();
    }
}
```

**Why this way:** overriding `source()` instead of folding the cause's
message into `Display` keeps the two concerns separate — the top-level
message stays short, and tools (or the manual chain-walk shown here) can
still reach the full cause chain, which is the purpose the
[`std::error::Error` docs](https://doc.rust-lang.org/std/error/trait.Error.html#method.source)
give for the method.

### Scenario: Designing a public API

A function that can fail for many unrelated reasons — parsing, I/O, a
third-party crate's own error type — doesn't want to invent one enum
covering all of them, so it returns a boxed trait object instead.

```
use std::error::Error;

fn load_and_parse(path: &str) -> Result<u32, Box<dyn Error>> {
    // <- `Box<dyn Error>`: accepts any concrete error type behind one boundary
    let contents = std::fs::read_to_string(path)?; // io::Error boxes automatically
    let value: u32 = contents.trim().parse()?; // ParseIntError boxes automatically too
    Ok(value)
}
```

**Why this way:** `Box<dyn Error>` is the standard escape hatch for an
application-level boundary that would rather not define a bespoke enum
for every combination of underlying failures — every concrete error type
`?` encounters converts into the box automatically, because
`Box<dyn Error>` implements `From<E>` for any `E: Error`, which is exactly
why `anyhow::Error` (an ergonomic wrapper over the same idea) is blessed
for application code under this scenario's crate policy.

### Scenario: Converting between types

Pairing a manual `Error` impl with a manual `From` impl is what lets `?`
convert an underlying error into a custom type without a derive macro
doing it implicitly.

```
use std::fmt;
use std::error::Error;

#[derive(Debug)]
enum CacheError {
    Backend(std::io::Error),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheError::Backend(e) => write!(f, "cache backend failed: {e}"),
        }
    }
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CacheError::Backend(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for CacheError { // <- written by hand, no #[from] derive involved
    fn from(e: std::io::Error) -> Self {
        CacheError::Backend(e)
    }
}

fn read_cache_entry(path: &str) -> Result<String, CacheError> {
    Ok(std::fs::read_to_string(path)?) // <- `?` uses the hand-written `From` impl above
}
```

**Why this way:** writing `From` by hand alongside a manual `Error` impl
shows exactly what `thiserror`'s `#[from]` attribute (see
[custom error types](custom-error-types.md)) generates — understanding
the manual mechanism makes the derive's shorthand legible rather than
magic, the same order the
[Rust Book](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
takes by showing manual error handling before reaching for crates that
automate it.

## Explanation (Embedded)

`std::error::Error` historically lived only in `std`, so a `#![no_std]`
crate either skipped implementing the standard trait entirely — defining
its own ad hoc "this is an error" shadow trait, or no trait at all — or
left that job to whatever `std`-level application eventually consumed it.
As the classic Explanation already notes, the trait has since been
stabilized in `core` as `core::error::Error`; the practical consequence for
embedded code is that a no_std error enum can now implement the *real*
`Error` trait directly, the same trait hosted code implements, with no
shadow substitute and no need to wait for a `std` boundary. `Display` and
`Debug`, the trait's two supertrait requirements, are core-language
already, so nothing about implementing `Error` itself needs an allocator —
the one piece that still does is `Box<dyn Error>`, which needs `alloc`
plus a `#[global_allocator]`. A no_std driver crate can implement `Error`,
and even override `source()`, without ever reaching for that: `source()`
can return a plain `&dyn Error` reference instead of a boxed one.

## Basic usage example (Embedded)

```
#![no_std]

use core::fmt;
use core::error::Error;

#[derive(Debug)]
struct BusFault;

impl fmt::Display for BusFault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sensor bus fault")
    }
}

impl Error for BusFault {} // <- the real core::error::Error, no shadow trait needed
```

## Best practices & deeper information (Embedded)

### Scenario: Handling and propagating errors

A sensor error wraps an underlying bus fault and overrides `source()` to
expose it — entirely without `Box`, so the whole cause chain stays
allocation-free.

```
use core::fmt;
use core::error::Error;

#[derive(Debug)]
struct BusFault;

impl fmt::Display for BusFault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bus fault")
    }
}

impl Error for BusFault {}

#[derive(Debug)]
struct SensorError {
    cause: BusFault,
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sensor read failed")
    }
}

impl Error for SensorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause) // <- a plain reference, not a Box: no alloc needed
    }
}
```

**Why this way:** overriding `source()` with a borrowed reference rather
than a boxed trait object keeps the entire cause chain allocation-free, so
a no_std driver gets the same "walk the cause chain" ergonomics as hosted
code without pulling in `alloc` just for error reporting.

### Scenario: Converting between types

Pairing a manual `Error` impl with a manual `From` impl is what lets `?`
convert a display's SPI failure into the driver's own error type, without
`thiserror`'s `#[from]` available to generate it.

```
use core::fmt;
use core::error::Error;

#[derive(Debug)]
struct SpiFault;

#[derive(Debug)]
enum DisplayError {
    Spi(SpiFault),
}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DisplayError::Spi(_) => write!(f, "display SPI transaction failed"),
        }
    }
}

impl Error for DisplayError {}

impl From<SpiFault> for DisplayError { // <- written by hand: no #[from] derive without thiserror
    fn from(e: SpiFault) -> Self {
        DisplayError::Spi(e)
    }
}

fn raw_spi_read() -> Result<u8, SpiFault> {
    Ok(0x42) // ... perform the actual SPI transfer
}

fn read_display_id() -> Result<u8, DisplayError> {
    let id = raw_spi_read()?; // <- `?` converts SpiFault into DisplayError via the From impl above
    Ok(id)
}
```

**Why this way:** with `thiserror` unavailable in a no_std crate, the
`From` impl has to be written by hand rather than generated by `#[from]` —
the same manual mechanism as the classic page's `CacheError` example, just
without `std::error::Error` requiring a `std` boundary to reach.

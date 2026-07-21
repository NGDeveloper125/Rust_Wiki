---
title: "The builder pattern"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns", "Design Patterns & Idioms", "Object-Oriented-ish Patterns", "Builders & Object Construction"]
related_syntax: []
see_also: ["The typestate pattern", "Compose structs", "Structs", "Ownership"]
---

## Explanation

The builder pattern constructs a value with many fields — especially one
where most fields are optional or have sane defaults — through a series
of chained method calls instead of one enormous constructor call or a
struct literal that forces every field to be spelled out at once. It
exists in Rust for a very concrete reason: the language has no method
overloading and no default parameter values, so a type with eight
optional settings can't get eight overloaded `new` functions the way a
Java or C++ constructor set might. A builder is Rust's answer, and it's
expressed as an ordinary struct plus ordinary methods — not a GoF-style
abstract `Builder` interface with one concrete subclass per variant.

Two shapes cover almost every case. A *consuming* builder has methods
that take `self` by value and return `Self`, so the whole thing reads as
one chained expression ending in `.build()`; this is the more common
shape, and it pairs naturally with [ownership](../ownership-borrowing/ownership.md)'s
idea of owned `self` meaning "this call consumes and reshapes the
value." A *non-consuming* builder takes `&mut self` and returns `&mut
Self` instead, which costs a little chaining ergonomics but lets the
caller hold the builder in a variable and call its methods conditionally
across branches or loops before finally building — something a
consuming builder, once moved into a method call, can't do.

A builder's `build()` step is also the natural place to enforce
invariants a plain struct literal can't: required fields can be stored
as `Option<T>` internally and checked only once, at `build()` time,
turning "forgot to set the URL" into a caught `Result::Err` instead of a
struct silently holding an empty string. Combined with the
[typestate pattern](the-typestate-pattern.md), a builder's own type can
even change as required fields are supplied, so code that skips a
mandatory field fails to *compile* rather than fails at `build()` — the
same idea taken one step further.

The builder pattern composes naturally with [structs](../types-data-modeling/structs.md)
as plain data holders and complements [compose structs](compose-structs.md):
a builder for a large, composed config type often just delegates to
smaller builders (or plain literals) for each sub-struct.

## Basic usage example

```
struct HttpRequestBuilder {
    method: String,
    url: String,
    timeout_secs: u32,
}

impl HttpRequestBuilder {
    fn new(url: &str) -> Self {
        Self { method: "GET".to_string(), url: url.to_string(), timeout_secs: 30 }
    }

    fn method(mut self, method: &str) -> Self { // <- consuming builder: takes and returns `Self` by value
        self.method = method.to_string();
        self
    }

    fn timeout_secs(mut self, secs: u32) -> Self {
        self.timeout_secs = secs;
        self
    }
}

let request = HttpRequestBuilder::new("https://example.com")
    .method("POST")
    .timeout_secs(10); // <- one chained expression builds the whole value
println!("{} {} ({}s)", request.method, request.url, request.timeout_secs);
```

## Best practices & deeper information

### Scenario: Creating a new object

A connection pool has one required setting (the host) and several that
are fine left at a default, which is exactly the shape a builder is for
— it beats a `Pool::new(host, port, max_connections, ...)` constructor
where callers have to pass every value even when they only care about
one.

```
struct PoolBuilder {
    host: String,
    port: u16,
    max_connections: u32,
}

impl PoolBuilder {
    fn new(host: &str) -> Self {
        Self { host: host.to_string(), port: 5432, max_connections: 10 }
    }

    fn max_connections(mut self, max: u32) -> Self { // <- builder pattern: override only what differs from the default
        self.max_connections = max;
        self
    }

    fn build(self) -> Pool {
        Pool { host: self.host, port: self.port, max_connections: self.max_connections }
    }
}

struct Pool {
    host: String,
    port: u16,
    max_connections: u32,
}

let pool = PoolBuilder::new("db.internal").max_connections(50).build();
println!("{}:{} (max {})", pool.host, pool.port, pool.max_connections);
```

**Why this way:** a builder lets every field default sensibly while
still letting a caller override exactly the ones that matter to them,
which is what the
[API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)'s
C-BUILDER item calls out builders for, and matches the
[Rust Design Patterns' builder entry](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html).

### Scenario: Validating input

A request without a URL isn't a request at all, so `build()` should
reject it with a `Result` rather than silently producing a value with an
empty string that fails much later, far from the actual mistake.

```
struct HttpRequestBuilder {
    url: Option<String>,
    method: String,
}

impl HttpRequestBuilder {
    fn new() -> Self {
        Self { url: None, method: "GET".to_string() }
    }

    fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    fn build(self) -> Result<HttpRequest, &'static str> {
        let url = self.url.ok_or("url is required")?; // <- required field enforced once, at build time
        Ok(HttpRequest { url, method: self.method })
    }
}

struct HttpRequest {
    url: String,
    method: String,
}

let request = HttpRequestBuilder::new().url("https://example.com").build().unwrap();
println!("{} {}", request.method, request.url);
```

**Why this way:** checking required fields exactly once, at construction,
is the [Effective Rust](https://effective-rust.com/) case for validating
at the earliest possible point — every later piece of code can then
trust `HttpRequest` is well-formed without re-checking it.

### Scenario: Designing a public API

A query builder that's assembled across several conditional branches
needs a non-consuming, `&mut self`-based chain, since a consuming
builder would be moved away the first time it's used inside an `if`.

```
#[derive(Default)]
struct QueryBuilder {
    filters: Vec<String>,
}

impl QueryBuilder {
    fn filter(&mut self, condition: &str) -> &mut Self { // <- &mut self: can be called conditionally without moving the builder
        self.filters.push(condition.to_string());
        self
    }

    fn build(&self) -> String {
        format!("WHERE {}", self.filters.join(" AND "))
    }
}

let mut query = QueryBuilder::default();
if true {
    query.filter("active = true"); // <- fine inside a branch: `query` was never consumed
}
query.filter("region = 'EU'");
println!("{}", query.build());
```

**Why this way:** picking `&mut self` over owned `self` is an explicit
API design decision about how the builder will be used — the
[Rust Design Patterns' builder entry](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
covers both variants and the tradeoff between chaining ergonomics and
conditional construction.

## Explanation (Embedded)

The builder pattern shows up constantly in embedded-hal-adjacent APIs for
exactly the reason it exists in hosted Rust: a peripheral config —
a UART's baud rate, parity, and stop-bits, a timer's prescaler and
counting mode, an SPI bus's clock polarity and phase — has several
independent settings, most of which have a sensible default (8N1 for a
UART, no parity) that only a minority of callers ever need to override.
A chained builder lets firmware code say `Uart::config().baud_rate(115_200).build()`
and get every other setting for free, instead of a constructor taking
every parameter positionally. The pattern itself costs nothing beyond the
fields the builder actually holds — no vtable, no allocation of its own
— so it's exactly as at-home in `#![no_std]` firmware as anywhere else.
The one thing worth watching is what the fields themselves are made of:
a builder over plain integers, bools, and small enums (baud rate, parity
mode) needs no allocator at all, while a builder that reaches for
`String`/`Vec<T>` fields for convenience pulls in the same `alloc`
requirement those types always carry — a `heapless::String`/`heapless::Vec<T, N>`
with a fixed capacity sidesteps that where the field's maximum size is
known up front.

## Basic usage example (Embedded)

```
struct UartConfigBuilder {
    baud_rate: u32,
    parity: bool,
    stop_bits: u8,
}

impl UartConfigBuilder {
    fn new() -> Self {
        Self { baud_rate: 9_600, parity: false, stop_bits: 1 } // sensible defaults: 8N1 at 9600 baud
    }

    fn baud_rate(mut self, baud_rate: u32) -> Self { // <- consuming builder: takes and returns `Self` by value
        self.baud_rate = baud_rate;
        self
    }
}

let config = UartConfigBuilder::new().baud_rate(115_200); // <- overrides only the setting that matters
```

## Best practices & deeper information (Embedded)

### Scenario: Creating a new object

A UART peripheral has one setting most firmware cares about (baud rate)
and two that are almost always left at their default (parity, stop
bits) — exactly the shape a builder is for, over a hand-written
`Uart::new(baud_rate, parity, stop_bits, ...)` that forces every caller
to spell out settings they don't care about.

```
struct UartConfig {
    baud_rate: u32,
    parity: bool,
    stop_bits: u8,
}

struct UartConfigBuilder {
    baud_rate: u32,
    parity: bool,
    stop_bits: u8,
}

impl UartConfigBuilder {
    fn new() -> Self {
        Self { baud_rate: 9_600, parity: false, stop_bits: 1 }
    }

    fn baud_rate(mut self, baud_rate: u32) -> Self { // <- builder pattern: override only what differs from the default
        self.baud_rate = baud_rate;
        self
    }

    fn build(self) -> UartConfig {
        UartConfig { baud_rate: self.baud_rate, parity: self.parity, stop_bits: self.stop_bits }
    }
}

let config = UartConfigBuilder::new().baud_rate(115_200).build();
```

**Why this way:** a builder lets every UART setting default sensibly
while still letting firmware override exactly the baud rate it needs,
which is what the
[API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)'s
C-BUILDER item calls out builders for — the same shape the
[Rust Design Patterns' builder entry](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
documents, unchanged by targeting a `#![no_std]` HAL.

### Scenario: Validating input

An SPI bus config without a clock frequency isn't a usable config at
all, so `build()` should reject it with a `Result` rather than silently
producing a config that would misconfigure the peripheral's clock
divider at init time.

```
struct SpiConfigBuilder {
    clock_hz: Option<u32>,
    mode: u8,
}

impl SpiConfigBuilder {
    fn new() -> Self {
        Self { clock_hz: None, mode: 0 }
    }

    fn clock_hz(mut self, clock_hz: u32) -> Self {
        self.clock_hz = Some(clock_hz);
        self
    }

    fn build(self) -> Result<SpiConfig, &'static str> {
        let clock_hz = self.clock_hz.ok_or("clock_hz is required")?; // <- required field enforced once, at build time
        Ok(SpiConfig { clock_hz, mode: self.mode })
    }
}

struct SpiConfig {
    clock_hz: u32,
    mode: u8,
}

let config = SpiConfigBuilder::new().clock_hz(1_000_000).build().unwrap();
```

**Why this way:** checking a required field like the SPI clock exactly
once, at construction, is the
[Effective Rust](https://effective-rust.com/) case for validating at the
earliest possible point — catching a missing clock setting here beats
discovering it only once the peripheral is already misbehaving on the
bus.

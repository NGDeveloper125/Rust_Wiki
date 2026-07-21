---
title: "Anti-pattern: Deref polymorphism (faking inheritance)"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Anti-patterns", "Design Patterns & Idioms", "Composition"]
related_syntax: ["*"]
see_also: ["Deref & DerefMut coercion", "Traits", "Dependency injection via traits/generics"]
---

## Explanation

[Deref & DerefMut coercion](../ownership-borrowing/deref-coercion.md)
covers what `Deref` is *for*: letting a smart pointer transparently act
like a reference to whatever it wraps, so `Box<String>` can be used
almost anywhere a `&String` is expected. This page is about a specific
misuse of that mechanism: implementing `Deref` on a wrapper type not to
make it act like a reference, but purely to make the wrapper "inherit"
every method of the type it wraps — a `Wrapper<Inner>` that implements
`Deref<Target = Inner>` so `wrapper.some_inner_method()` compiles via
autoderef, imitating the way a subclass in an object-oriented language
inherits its parent's methods.

It's tempting because it genuinely works and requires almost no code: one
`impl Deref for Wrapper` block, and every public method of `Inner`
suddenly appears to be a method of `Wrapper` too, without writing a
single forwarding function by hand. For someone thinking in terms of
class inheritance, it looks like exactly the tool for the job.

The trouble is that `Deref` was never designed to mean "is-a" — it means
"acts like a reference to." Stretching it to fake inheritance creates
real problems: every method of `Inner` leaks into `Wrapper`'s public
surface whether or not that's actually wanted, so the wrapper's true API
becomes whatever `Inner` happens to expose, plus whatever methods
`Wrapper` adds — which can silently shadow an `Inner` method of the same
name in confusing ways. It also breaks down the moment `Wrapper` legitimately
needs to diverge from `Inner`'s behavior for one of those methods,
because autoderef doesn't know to prefer `Wrapper`'s own logic over a
method resolution that quietly reaches through to `Inner`.

The idiomatic alternative is explicit delegation: write forwarding
methods by hand for exactly the operations `Wrapper` intends to expose
(a handful of lines, ideally with each doing real work — validating,
logging, adjusting an argument — rather than being pure boilerplate), or
have both `Wrapper` and `Inner` implement a shared
[trait](../traits-polymorphism/traits.md) that names the behavior they
have in common. Either way, `Wrapper`'s public API is exactly what its
author wrote, not an accidental byproduct of what `Inner` happens to
expose — composition, not inherited method access via a pointer
coercion.

## Basic usage example

```
struct Meters(f64);

impl Meters {
    fn to_feet(&self) -> f64 { // <- PREFER: an explicit method, not an inherited f64 method via Deref
        self.0 * 3.28084
    }
}

let distance = Meters(10.0);
println!("{:.2} ft", distance.to_feet());
```

## Best practices & deeper information

### Scenario: Designing a public API

A logging wrapper around a database connection wants callers to run
queries through it (so it can log them), not bypass logging by reaching
straight through to every method the inner connection happens to expose.

```
struct DbConnection;

impl DbConnection {
    fn query(&self, sql: &str) -> Vec<String> {
        vec![format!("row for: {sql}")]
    }

    fn execute(&self, sql: &str) {
        println!("executed: {sql}");
    }
}

// AVOID: Deref makes every DbConnection method available on LoggingConnection unintentionally
// struct LoggingConnection {
//     inner: DbConnection,
// }
//
// impl std::ops::Deref for LoggingConnection {
//     type Target = DbConnection;
//     fn deref(&self) -> &DbConnection {
//         &self.inner // now `logging_conn.execute(..)` silently skips logging entirely
//     }
// }

// PREFER: explicit delegation — only the methods the wrapper actually intends to expose, each doing its own job
struct LoggingConnection {
    inner: DbConnection,
}

impl LoggingConnection {
    fn query(&self, sql: &str) -> Vec<String> {
        println!("[log] query: {sql}");
        self.inner.query(sql) // <- forwards deliberately, with logging attached
    }

    fn execute(&self, sql: &str) {
        println!("[log] execute: {sql}");
        self.inner.execute(sql);
    }
}

let conn = LoggingConnection { inner: DbConnection };
conn.query("SELECT 1"); // always logged: there's no bypass path through Deref
```

**Why this way:** implementing `Deref` on `LoggingConnection` would make
`execute` (and any future `DbConnection` method) reachable straight
through the wrapper without ever going through the wrapper's own logic,
silently defeating the entire point of wrapping it; the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html)
documents "Deref polymorphism" as exactly this misuse, and recommends
explicit delegation or a shared trait instead.

## Explanation (Embedded)

The anti-pattern is identical to the classic case, and the fix is too:
`Deref` still means "acts like a reference to," never "is-a," regardless
of target. The embedded flavor worth calling out is where the temptation
shows up most: wrapping an `embedded-hal` peripheral handle — an `I2c`, a
`Spi`, a GPIO `OutputPin` — to add behavior the raw HAL type doesn't have
(retrying a flaky I2C transaction, logging every register write,
debouncing a pin read). Implementing `Deref` on that wrapper so the
underlying HAL type's methods stay callable through it is exactly as
tempting here as with a database connection, and exactly as wrong: it
lets application code silently reach straight through the wrapper and
call the raw HAL method directly, skipping the retry/logging/debounce
logic the wrapper exists to add — on hardware, that's not just a
surprising API, it can mean a transaction that's supposed to be retried
on a noisy bus silently isn't, because half the codebase called the
un-wrapped method by accident via autoderef.

## Basic usage example (Embedded)

```
struct RetryingI2c<I> {
    inner: I,
    max_attempts: u8,
}

impl<I> RetryingI2c<I> {
    fn attempts(&self) -> u8 { // <- PREFER: an explicit method, not an inherited I2c method via Deref
        self.max_attempts
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL wrapper around an I2C bus wants every read to retry on a bus
error; callers must go through the wrapper's own method, never straight
through to the raw HAL implementation underneath it.

```
trait I2cRead {
    fn read(&mut self, addr: u8, buf: &mut [u8]) -> Result<(), ()>;
}

struct RawI2c;
impl I2cRead for RawI2c {
    fn read(&mut self, _addr: u8, _buf: &mut [u8]) -> Result<(), ()> {
        Err(()) // stands in for a flaky bus transaction
    }
}

// AVOID: Deref makes the raw, non-retrying `read` reachable straight through the wrapper
// struct RetryingI2c<I> {
//     inner: I,
// }
//
// impl<I> std::ops::Deref for RetryingI2c<I> {
//     type Target = I;
//     fn deref(&self) -> &I {
//         &self.inner // now `wrapper.read(..)` silently skips the retry loop entirely
//     }
// }

// PREFER: explicit delegation — the only `read` callers can reach is the retrying one
struct RetryingI2c<I> {
    inner: I,
    max_attempts: u8,
}

impl<I: I2cRead> RetryingI2c<I> {
    fn read(&mut self, addr: u8, buf: &mut [u8]) -> Result<(), ()> {
        for _ in 0..self.max_attempts {
            if self.inner.read(addr, buf).is_ok() {
                return Ok(());
            }
        }
        Err(())
    }
}

let mut sensor_bus = RetryingI2c { inner: RawI2c, max_attempts: 3 };
let mut buf = [0u8; 2];
let _ = sensor_bus.read(0x48, &mut buf); // <- always goes through the retry loop
```

**Why this way:** implementing `Deref` on `RetryingI2c` would make the
raw, non-retrying `read` reachable through autoderef alongside the
wrapper's own method, so a single unretried call at the wrong moment on
a noisy I2C bus can silently corrupt a sensor read that the wrapper was
specifically built to make reliable; the
[Rust Design Patterns' anti-patterns section](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html)
documents this "Deref polymorphism" misuse and recommends explicit
delegation instead — doubly so where the delegated behavior (a retry
loop) is the entire reason the wrapper exists.

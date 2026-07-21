---
title: "Constructor functions (new() convention)"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Builders & Object Construction"]
related_syntax: [fn]
see_also: ["The Default trait as idiom", "Privacy for extensibility", "Structs", "Visibility & privacy (pub and friends)"]
---

## Explanation

Rust has no `new` keyword and no built-in notion of a constructor — a
type is just data, and producing one is just a function call like any
other. The convention that fills this gap is a plain associated function
named `new`, defined in the type's own `impl` block, that builds and
returns an instance of `Self`. Because it's ordinary code rather than
compiler-recognized syntax, nothing stops a type from having several
constructors side by side (`new`, `with_capacity`, `from_bytes`), each
named for what it does rather than overloaded under one reserved word —
something languages with a real `new` keyword and constructor overloading
can't offer as cleanly.

The convention carries real expectations, not just a naming habit. A
function called `new` is expected to be infallible and non-consuming: it
takes whatever inputs it needs and always succeeds in producing a value,
with no `Result` and no borrowed argument it fails to give back. When
construction can fail — a port number that's out of range, a string that
isn't valid UTF-8 — the idiomatic move is a differently-named function
returning `Result<Self, E>` (`Config::from_file`, `Port::try_new`), so the
fallibility is visible in both the name and the signature rather than
hidden behind a name that promises "this always works."

`new` also pairs naturally with keeping a type's fields private (see
[Privacy for extensibility](privacy-for-extensibility.md)): once outside
code can't build the type with a struct literal, `new` (or whichever
constructor a type exposes) becomes the *only* path to an instance, which
means it's also the only place that has to enforce the type's invariants.
Everything downstream of construction can then simply trust that an
existing value is valid, because there was never a way to build an
invalid one.

Where a type genuinely has no required inputs, [the `Default`
trait](the-default-trait-as-idiom.md) often takes over `new`'s job
instead of duplicating it — the two idioms are closely related and the
API Guidelines recommend implementing both when a zero-argument `new`
exists, with `new` delegating to `Default::default()` (or vice versa) so
the two never drift apart.

## Basic usage example

```
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u64,
}

impl RetryPolicy {
    fn new(max_attempts: u32, backoff_ms: u64) -> Self { // <- constructor: builds and returns Self, always succeeds
        RetryPolicy { max_attempts, backoff_ms }
    }
}

let policy = RetryPolicy::new(3, 250);
println!("{} attempts", policy.max_attempts);
```

## Best practices & deeper information

### Scenario: Creating a new object

A connection pool has one truly required input — how many connections to
keep open — so `new` takes exactly that and nothing else, leaving room
for a separate constructor later if more configuration knobs show up.

```
struct ConnectionPool {
    size: usize,
    connections: Vec<u32>, // stand-in for real connection handles
}

impl ConnectionPool {
    fn new(size: usize) -> Self { // <- new(): the minimal, infallible constructor
        ConnectionPool {
            size,
            connections: (0..size as u32).collect(),
        }
    }
}

let pool = ConnectionPool::new(8);
println!("pool has {} connections", pool.connections.len());
```

**Why this way:** `new` taking only the genuinely required parameters and
always succeeding is the convention readers expect from the name itself;
the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/ctor.html)
book documents this exact shape as the idiomatic Rust constructor.

### Scenario: Designing a public API

A parser can be built from different sources — a file path, a raw
string, a set of already-parsed tokens — so instead of overloading one
`new`, each source gets its own clearly-named constructor.

```
struct Config {
    entries: Vec<(String, String)>,
}

impl Config {
    fn new() -> Self { // <- new(): the empty, always-valid starting point
        Config { entries: Vec::new() }
    }

    fn from_str(source: &str) -> Self { // <- named for its source, not overloaded under `new`
        let entries = source
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config { entries }
    }
}

let empty = Config::new();
let loaded = Config::from_str("host=localhost\nport=8080");
println!("{} {}", empty.entries.len(), loaded.entries.len());
```

**Why this way:** without constructor overloading, giving each
construction path its own name keeps every one of them self-documenting
at the call site — the
[API Guidelines' constructor conventions](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
recommend exactly this: static, inherent methods named for what they
build from, with `new` reserved for the no-argument-or-obvious-argument
case.

## Explanation (Embedded)

The convention is identical — `new` as a plain associated function
returning `Self`, expected to be infallible — and it shows up in
embedded driver code in one particularly common shape: a constructor
that takes ownership of the underlying HAL handle it will exclusively
control. `Sensor::new(i2c: I2c) -> Self` doesn't just store a reference
to the I2C peripheral, it *consumes* it — the caller no longer has the
`I2c` value once `Sensor::new` returns, only the `Sensor` that now owns
it. This isn't a stylistic preference; it's the mechanism embedded Rust
uses to enforce, at compile time, that no two drivers can independently
try to drive the same physical bus. If `new` only borrowed the handle,
two sensor drivers could each hold a `&I2c` to the same peripheral and
issue conflicting transactions with no compiler complaint; taking
ownership means only one driver can exist per handle, ever, because the
type system tracks the handle as moved.

Where construction can genuinely fail — the sensor doesn't answer at the
expected I2C address, a self-test byte doesn't match — the same
fallible-constructor convention from the classic page applies: `try_new`
(or `Sensor::new` returning `Result<Self, SensorError>`) rather than a
`new` that silently returns a driver wired to a device that isn't
actually there.

## Basic usage example (Embedded)

```
struct I2c; // stands in for a HAL I2c peripheral handle

struct Sensor {
    i2c: I2c,
}

impl Sensor {
    fn new(i2c: I2c) -> Self { // <- takes ownership of the HAL handle: only one Sensor can exist per bus
        Sensor { i2c }
    }
}

let bus = I2c;
let sensor = Sensor::new(bus); // <- `bus` is moved; nothing else can construct a driver on it now
```

## Best practices & deeper information (Embedded)

### Scenario: Creating a new object

A sensor may not actually be present on the bus at the address the
driver expects, so its constructor probes for it and reports failure
instead of silently returning a `Sensor` that will fail on its first real
read.

```
struct I2c;
impl I2c {
    fn probe(&self, address: u8) -> bool {
        address == 0x76 // stands in for a real device-ID register check
    }
}

struct SensorError;

struct Sensor {
    i2c: I2c,
    address: u8,
}

impl Sensor {
    fn try_new(i2c: I2c, address: u8) -> Result<Self, SensorError> { // <- fallible construction: named accordingly, not called `new`
        if !i2c.probe(address) {
            return Err(SensorError);
        }
        Ok(Sensor { i2c, address })
    }
}

let bus = I2c;
match Sensor::try_new(bus, 0x76) {
    Ok(sensor) => println!("sensor ready at 0x{:02x}", sensor.address),
    Err(_) => println!("no sensor responded"),
}
```

**Why this way:** naming the fallible path `try_new` instead of `new`
keeps the promise "a function called `new` always succeeds" intact,
which matters more in firmware than in a desktop app — a `Sensor::new`
that silently constructs a driver bound to a device that never answers
would only surface the mistake later, likely at the first real read, far
from the constructor that caused it; the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/ctor.html)
convention of a differently-named fallible constructor applies here
unchanged.

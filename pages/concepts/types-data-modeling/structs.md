---
title: "Structs"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Object-Oriented-ish Patterns", "Designing Robust Data Models", "Composition"]
related_syntax: [struct]
see_also: ["Tuple structs", "Unit structs", "Enums (algebraic data types)"]
---

## Explanation

A struct groups related values together under named fields into a single
type — the basic building block for modeling data with meaning attached
to each piece, rather than passing several loose, unrelated parameters
around; for example, `struct Point { x: f64, y: f64 }` gives a pair of
coordinates a name and a shared identity instead of two bare `f64`s.

Structs are Rust's primary way to give a related bundle of data its own
identity and its own methods (via `impl` blocks), filling a role similar
to a class in an object-oriented language but without inheritance —
behavior is attached through trait implementations and composition (a
struct containing other structs) rather than an inheritance hierarchy.
This composition-first approach is a deliberate design choice: nesting
structs inside each other, and implementing shared traits across
otherwise-unrelated types, covers most of what inheritance is used for in
other languages, without the fragility multi-level inheritance
hierarchies tend to accumulate over time.

## Basic usage example

```
struct Point { // <- defines the struct: a new type with two named fields
    x: f64,
    y: f64,
}

let p = Point { x: 1.0, y: 2.0 }; // constructing a value of that type
println!("{}", p.x); // fields are accessed by name
```

## Best practices & deeper information

### Scenario: Creating a new object

A struct with several fields is usually built through a `new()`
constructor rather than a raw struct literal at every call site, so
required setup (defaults, normalization) lives in one place.

```
struct Order {
    id: u64,
    customer: String,
    total_cents: u64,
}

impl Order {
    fn new(id: u64, customer: impl Into<String>, total_cents: u64) -> Self {
        // <- `new` is the constructor convention: an inherent, static method
        Order { id, customer: customer.into(), total_cents }
    }
}

let order = Order::new(1, "Alice", 4599);
```

**Why this way:** the API Guidelines call this out directly as
[C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor) —
constructors are static, inherent methods named `new`, which lets a type
add defaulting or validation later without breaking every existing
struct-literal call site.

### Scenario: Querying a database

`#[derive(sqlx::FromRow)]` maps a query's result columns onto a struct's
fields by name, so the type that models the domain object is also what
the query returns — no manual `row.get("column")` calls to keep in sync.

```
// [dependencies] sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }, tokio = { version = "1", features = ["full"] }
#[derive(sqlx::FromRow)] // <- maps each returned row's columns onto this struct's fields
struct Order {
    id: i64,
    customer: String,
    total_cents: i64,
}

async fn fetch_order(pool: &sqlx::PgPool, id: i64) -> sqlx::Result<Order> {
    sqlx::query_as::<_, Order>("SELECT id, customer, total_cents FROM orders WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}
```

**Why this way:** deriving `FromRow` on the same struct used everywhere
else in the code keeps one type as the single source of truth for what an
order looks like, instead of a second ad-hoc "row" shape that has to be
converted by hand — see
[sqlx's `FromRow` docs](https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html).

### Scenario: Serializing and deserializing

The same kind of struct that models a database row can derive
`serde::Serialize`/`Deserialize` to move across a JSON boundary, with the
struct's field names becoming the JSON object's keys automatically.

```
// [dependencies] serde = { version = "1", features = ["derive"] }, serde_json = "1"
#[derive(serde::Serialize, serde::Deserialize)] // <- struct fields become JSON keys automatically
struct Order {
    id: u64,
    customer: String,
    total_cents: u64,
}

let order = Order { id: 1, customer: "Alice".into(), total_cents: 4599 };
let json = serde_json::to_string(&order).unwrap();
let parsed: Order = serde_json::from_str(&json).unwrap(); // <- round-trips through the same struct
```

**Why this way:** deriving on the struct definition itself keeps the wire
format and the Rust type in sync automatically — renaming or adding a
field updates both sides at once, which
[serde's own guide](https://serde.rs/derive.html) recommends over
hand-written (de)serialization for exactly this reason.

## Explanation (Embedded)

A struct's job in embedded Rust is the same job it has everywhere else —
giving a bundle of related data one identity — but the "related data" a
driver needs to track is usually a mix of a peripheral handle, calibration
or configuration values, and cached state, and a struct is what lets all
three move and borrow as a single unit. A temperature-sensor driver, for
instance, typically needs to hold onto the I2C peripheral it talks over, a
calibration offset read once at startup, and the last reading it took (so
a caller can ask "what was it last time" without re-triggering a bus
transaction) — three pieces of state that belong together conceptually
and, because they're fields of one struct, move together, get exclusive
`&mut` access together, and get dropped together. Without the struct, a
function juggling a peripheral handle, a calibration value, and a cache as
three separate parameters would have no way to express that they're one
logical unit — and no way for the borrow checker to enforce that only one
part of the program touches the sensor's state at a time.

This is a distinct concern from the register-block layout question
covered on the [`struct` syntax page](../../syntax/keywords/struct.md) and
[`#[repr(...)]`](../../syntax/attributes/repr.md): those pages cover
pinning a struct's field layout to match a hardware memory map so it can
overlay memory-mapped registers byte-for-byte. This page's struct — the
driver wrapping a peripheral — usually contains one of those register-block
structs (or a PAC-generated handle to one) as a field, but the driver
struct itself has no memory-mapped layout to preserve; it's an ordinary
`#[repr(Rust)]` struct doing ordinary struct-grouping work, just with
embedded-flavored fields.

## Basic usage example (Embedded)

```
struct TemperatureSensor<I2C> {
    i2c: I2C,                  // <- the peripheral handle this driver owns
    calibration_offset: i16,   // <- state read once at startup
    last_reading: Option<f32>, // <- cache: avoids re-reading the bus unnecessarily
}
```

## Best practices & deeper information (Embedded)

### Scenario: Creating a new object

A driver's constructor is where the peripheral handle and any calibration
data are gathered up once, so the rest of the driver's methods never have
to wonder whether calibration happened yet.

```
struct TemperatureSensor<I2C> {
    i2c: I2C,
    calibration_offset: i16,
    last_reading: Option<f32>,
}

impl<I2C> TemperatureSensor<I2C> {
    fn new(i2c: I2C, calibration_offset: i16) -> Self { // <- `new` gathers peripheral + calibration in one place
        TemperatureSensor { i2c, calibration_offset, last_reading: None }
    }
}
```

**Why this way:** the same
[C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
convention that applies to hosted Rust applies here — a constructor is the
one place calibration setup can be validated or normalized, instead of
every call site being responsible for assembling a correctly-calibrated
driver by hand.

### Scenario: Designing a public API

Keeping the peripheral handle and calibration data private, and exposing
only a narrow read method, stops calling code from bypassing the driver's
own bookkeeping (like the reading cache) by reaching into the struct
directly.

```
pub struct TemperatureSensor<I2C> {
    i2c: I2C,                  // <- private: calling code can't bypass the driver's own bookkeeping
    calibration_offset: i16,   // <- private: calibration is an implementation detail
    last_reading: Option<f32>,
}

impl<I2C> TemperatureSensor<I2C> {
    pub fn last_reading(&self) -> Option<f32> { // <- narrow, public accessor instead of public fields
        self.last_reading
    }
}
```

**Why this way:** private fields plus narrow accessors keep the
calibration offset and the cache internally consistent — a caller who
could write `last_reading` directly could desynchronize the cache from
what the sensor actually reported, which is exactly the class of bug
encapsulation exists to prevent.

### Scenario: Managing resources (RAII)

A driver that owns a peripheral is also the natural place to put shutdown
behavior — implementing `Drop` on the struct guarantees the peripheral
gets powered down or released whenever the driver goes out of scope,
including on an early return.

```
struct TemperatureSensor<I2C> {
    i2c: I2C,
    powered_on: bool,
}

impl<I2C> Drop for TemperatureSensor<I2C> {
    fn drop(&mut self) { // <- runs whenever the driver goes out of scope, including on early return
        self.powered_on = false; // real code would issue a power-down command over `i2c` here
    }
}
```

**Why this way:** tying peripheral shutdown to the struct's `Drop` impl
means every path out of scope — a normal return, an early `?`, a panic
during unwinding — reliably powers the sensor down, instead of relying on
every caller to remember an explicit `shutdown()` call.

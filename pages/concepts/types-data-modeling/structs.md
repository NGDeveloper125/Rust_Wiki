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
around.

```
struct Point {
    x: f64,
    y: f64,
}
```

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

## Embedded Rust Notes

**Full support.** Structs are core-language and allocator-free — the
primary way embedded HAL crates model peripherals, register blocks, and
driver state.

---
title: "struct"
kind: keyword
embedded_support: full
groups: ["Types & Data Structures", Basics]
related_concepts: [Structs, "Tuple structs", "Unit structs"]
related_syntax: ["{ }", "( )", ";", ":"]
see_also: [enum]
---

## Explanation

`struct` declares a new named product type and comes in three distinct
grammar forms, distinguished by what follows the type's name.

The **named-field** form opens a brace block immediately after the name:
`struct Point { x: f64, y: f64 }`. Each field is `name: Type`, separated
by commas, with an optional trailing comma before the closing brace; the
block itself ends the item, so no semicolon follows it. The **tuple**
form instead opens a parenthesized list of bare types with no field
names — `struct Meters(f64);` — and, because parentheses don't close the
item the way a brace block does, a trailing semicolon is required. The
**unit** form has neither: just the name and a semicolon, `struct
Marker;`. All three accept generic parameters and a `where` clause in the
same position, right after the name and before whichever field syntax
follows: `struct Wrapper<T> where T: Clone { value: T }`.

Field visibility is controlled per field with `pub` (or a restricted
`pub(crate)`/`pub(in path)`) directly before the field's name, in both
the named and tuple forms — `pub x: f64` or `pub Meters(pub f64)`.
Deriving standard traits is written as one or more `#[derive(...)]`
attributes placed directly above the `struct` item, before any doc
comments or other attributes are typically irrelevant to ordering.
Forgetting the trailing `;` on a tuple or unit struct, or adding one after
a named-field struct's closing `}`, are the two most common syntax slips
— the compiler's error message names the exact fix in both cases.

Which of the three forms to reach for, and why, is a design question
answered on the [Structs](../../concepts/types-data-modeling/structs.md),
[Tuple structs](../../concepts/types-data-modeling/tuple-structs.md), and
[Unit structs](../../concepts/types-data-modeling/unit-structs.md)
concept pages; this page covers only the grammar each form requires.

## Usage examples

### Declaring and instantiating a named-field struct

```
struct Point { x: f64, y: f64 } // <- `struct` declares a named-field type; no `;` after the `}`

let p = Point { x: 1.0, y: 2.0 };
println!("{}", p.x);
```

### Creating a new object

Constructing a value of a named-field struct uses the same `Name { field:
value, ... }` syntax the declaration introduced; the field-init shorthand
and struct-update syntax (`..`) both reuse that same literal form.

```
struct Order {
    id: u64,
    customer: String,
    total_cents: u64,
}

let id = 42;
let base = Order { id, customer: "Alice".into(), total_cents: 1500 };
// <- `id` alone is shorthand for `id: id`, legal because the local name matches the field name

let reordered = Order { total_cents: 2200, ..base }; // <- `..base` fills every other field from `base`
```

The field-init shorthand and `..` update syntax both
come directly from the struct declaration's field list, so renaming a
field is a single edit that the compiler will flag everywhere the old
name was still expected — clippy's
[`redundant_field_names`](https://rust-lang.github.io/rust-clippy/master/#redundant_field_names)
lint nudges toward the shorthand form whenever the names already match.

### Branching on data (pattern matching)

The same field list a `struct` declares can be destructured in a `let` or
`match` pattern, binding each field to a local name in one step instead
of reading it back out with `.field` afterward.

```
struct Reading {
    sensor_id: u32,
    celsius: f64,
}

let reading = Reading { sensor_id: 7, celsius: 21.5 };
let Reading { sensor_id, celsius } = reading; // <- destructures using the same field names from the struct's declaration
println!("sensor {sensor_id}: {celsius}°C");

// a `..` in a pattern ignores the remaining fields explicitly:
let Reading { celsius, .. } = Reading { sensor_id: 8, celsius: 19.0 };
```

Naming only the fields a function actually needs, with
`..` for the rest, keeps the pattern resilient to a struct gaining new
fields later — a bare `let Reading { sensor_id, celsius } = reading;`
would stop compiling the moment a third field is added, forcing every
destructuring site to be revisited.

## Explanation (Embedded)

This is where `struct` becomes one of the defining idioms of embedded
Rust, rather than merely "still works." Tools like svd2rust read a chip
vendor's SVD file — an XML description of every peripheral's registers,
their byte offsets, and bit layouts — and generate one `#[repr(C)]`
struct per peripheral, its fields laid out in declaration order to match
the hardware's memory map exactly. Each field is typically a thin,
interior-mutable register-cell type sitting at that field's exact offset,
so `peripheral.cr1.write(|w| ...)` compiles down to a single volatile
store at the register's real address. `#[repr(C)]` matters more here than
in ordinary FFI: Rust's default field ordering and padding are
unspecified, and a register that lands a few bytes off its real hardware
address is a silent wrong-memory-access bug, not a compile error. Pairing
one `struct` per peripheral with Rust's ownership rules — only one
`&mut` to a peripheral's register struct at a time — is also how
embedded Rust turns "only one part of the program may be programming
this peripheral right now" into a compile-time guarantee instead of a
runtime convention.

## Usage examples (Embedded)

### A register-block struct matching a hardware memory map

```
#[repr(C)] // <- guarantees field order/offsets match the hardware layout, not compiler-chosen layout
struct Gpio { // <- `struct` models the peripheral's registers, one field per register
    moder: u32,   // offset 0x00: mode register
    otyper: u32,  // offset 0x04: output type register
    ospeedr: u32, // offset 0x08: output speed register
    idr: u32,     // offset 0x0C: input data register
    odr: u32,     // offset 0x10: output data register
}

fn set_pin_high(gpio: &mut Gpio, pin: u32) {
    gpio.odr |= 1 << pin;
}
```

### Bundling driver state alongside a peripheral handle

A driver crate wraps the peripheral it owns together with any state the
driver itself needs to track, so the whole thing moves and borrows as one
unit.

```
struct TemperatureSensor<I2C> { // <- `struct` bundles a peripheral handle with driver-owned state
    i2c: I2C,
    address: u8,
    last_reading: Option<f32>,
}
```

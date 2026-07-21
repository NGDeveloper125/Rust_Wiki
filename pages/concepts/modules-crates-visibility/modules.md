---
title: "Modules"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project", "Encapsulation"]
related_syntax: [mod, use, super, crate, pub]
see_also: ["Crates", "Visibility & privacy (pub and friends)"]
---

## Explanation

A module is a named container for items — functions, structs, enums,
traits, constants, even other modules — that gives a crate its own
internal namespace. Rather than every function and type in a program
living in one flat pool of names, `mod` lets related items be grouped
together and addressed through a path, the same way a filesystem groups
files into directories. Modules are purely an organizational tool inside
a single crate; the crate itself (see [Crates](crates.md)) is the unit
the compiler actually compiles and links.

The mental model is a tree: every crate has one root module (the crate
root — the top of `main.rs` or `lib.rs`), and every `mod` declaration
adds a branch beneath whichever module it's written in. Items are then
addressed by walking that tree with `::`, e.g. `inventory::restock`, the
same way a path walks directories. Every branch in that tree is also a
privacy boundary: by default, an item is visible only inside the module
that defines it and that module's descendants, which is what makes
modules the mechanism behind [visibility & privacy](visibility-and-privacy.md)
rather than a separate, unrelated feature.

A `mod foo;` declaration (as opposed to `mod foo { ... }` written inline)
tells the compiler to load that module's contents from another file —
`foo.rs`, or `foo/mod.rs` for a module that itself has submodules — so a
crate's on-disk directory layout can mirror its logical module tree. This
is why a well-organized crate's source tree often reads like a table of
contents: `src/parser.rs`, `src/render/mod.rs`, `src/render/svg.rs`, each
file a module, each nesting level a deeper namespace.

`use` is the other half of the picture: it doesn't create anything, it
just brings a path into scope so it can be written shorter afterward.
Combined with re-exports (`pub use`), a crate can organize its
implementation into many small, private internal modules while still
presenting a small, flat, curated set of public paths to its callers —
the module tree used for organizing code doesn't have to match the
module tree exposed as the public API.

Modules stop at the crate boundary — there's no way for a module in one
crate to declare a submodule that actually lives inside another crate.
Once a set of modules grows to the point where it deserves independent
compilation, versioning, or publishing, that's the signal to split it
into its own crate, and, if several such crates need to be developed
side by side, to group them under a [workspace](workspaces.md).

## Basic usage example

```
mod inventory {                          // <- a module groups related items together
    pub struct Item {
        pub name: String,
        pub quantity: u32,
    }

    pub fn restock(item: &mut Item, amount: u32) {
        item.quantity += amount;
    }
}

use inventory::Item;                     // <- brings the nested path into scope

fn main() {
    let mut widget = Item { name: "Widget".into(), quantity: 3 };
    inventory::restock(&mut widget, 5);   // <- still reachable through its full path too
    println!("{} x{}", widget.name, widget.quantity);
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A library crate's internal module layout doesn't have to match what
callers see: keeping implementation modules private and re-exporting a
curated set of paths from the crate root lets internals move around
freely without breaking anyone's imports.

```
// src/lib.rs
mod parser;                  // <- private module: an implementation detail
mod render;                  // <- also private

pub use parser::Parser;      // <- re-export: reachable as `crate::Parser` despite `parser` being private
pub use render::Renderer;    // <- callers never need to know the internal module names

// src/parser.rs
pub struct Parser {
    // ...
}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }
}
```

**Why this way:** a flat, curated set of `pub use` re-exports at the crate
root gives callers a small, stable surface to depend on, while the actual
module boundaries underneath stay free to be refactored — this is
standard practice for organizing a crate's public API, per the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html).

### Scenario: Testing

Unit tests conventionally live in a nested `tests` module in the same
file as the code they exercise, compiled only when running `cargo test`.

```
pub fn discount_price(price_cents: u32, percent_off: u8) -> u32 {
    price_cents - (price_cents * percent_off as u32 / 100)
}

#[cfg(test)]
mod tests {                  // <- a nested module, compiled only for `cargo test`
    use super::*;             // <- pulls the parent module's items into scope

    #[test]
    fn applies_percentage_discount() {
        assert_eq!(discount_price(1000, 25), 750);
    }
}
```

**Why this way:** because a child module can always see its ancestors'
private items, `tests` can exercise `discount_price` and any other
private helpers in the same file without anything being made `pub` just
for the tests' sake — the layout the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
uses throughout.

## Explanation (Embedded)

Modules are a purely compile-time organizational construct with no
runtime representation, so the mechanism is unchanged under
`#![no_std]`. The typical structure is worth grounding concretely: a HAL
crate's `lib.rs` commonly declares one module per peripheral family —
`gpio`, `spi`, `i2c`, `uart` — mirroring the chip's own peripheral set,
each module wrapping that peripheral's register block in a small,
focused API before the crate re-exports a curated set of types drawn
from each of them.

## Basic usage example (Embedded)

```
// hal/src/lib.rs
mod gpio;   // <- src/gpio.rs: general-purpose I/O pins
mod spi;    // <- src/spi.rs: SPI peripheral driver
mod i2c;    // <- src/i2c.rs: I2C peripheral driver
mod uart;   // <- src/uart.rs: UART peripheral driver

pub use gpio::Pin;
pub use spi::Spi;
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A HAL crate's peripheral modules (`gpio`, `spi`, `i2c`, `uart`) each keep
their own register-level details private, and the crate root curates one
flat, public set of types drawn from all of them.

```
// hal/src/lib.rs
mod gpio;  // <- private module: internal register layout stays hidden
mod spi;
mod i2c;
mod uart;

pub use gpio::Pin;    // <- curated re-export: callers use `hal::Pin`, never `hal::gpio::Pin`
pub use spi::Spi;
pub use i2c::I2c;
pub use uart::Uart;
```

**Why this way:** re-exporting a flat set of driver types from the crate
root, rather than making callers dig into `hal::gpio::Pin`, keeps the
HAL's internal peripheral-module layout free to change as chip variants
are added, the same curation the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends generally.

### Scenario: Testing

A HAL's `i2c` module mixes register-twiddling logic with pure
protocol-encoding logic (building the byte sequence for a given
command); splitting the pure part into its own function lets it be
unit-tested on the host, without any real I2C bus involved.

```
// hal/src/i2c.rs
pub fn encode_write_command(register: u8, value: u8) -> [u8; 2] {  // <- pure logic: no register access
    [register, value]
}

pub fn write_register(&mut self, register: u8, value: u8) {  // <- touches real hardware, not unit-tested here
    let bytes = encode_write_command(register, value);
    // ... unsafe register/bus write using `bytes`
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_register_and_value_as_two_bytes() {
        assert_eq!(encode_write_command(0x6B, 0x00), [0x6B, 0x00]);
    }
}
```

**Why this way:** unit tests for a `#![no_std]` crate still compile and
run for the host by default (`cargo test` targets the host unless
configured otherwise), so isolating protocol/encoding logic from actual
register access into its own module-level function is what makes that
logic testable at all without hardware in the loop, an approach the
[Embedded Rust Book](https://docs.rust-embedded.org/book/) recommends
for driver crates.

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

## Embedded Rust Notes

**Full support.** Modules are a purely compile-time organizational
construct with no runtime representation — they cost nothing and require
no allocator or OS, so the module tree of a `#![no_std]` crate works
exactly the same as a hosted one. Embedded codebases lean on this just as
much as any other Rust project, often more: splitting hardware drivers,
protocol parsing, and application logic into separate modules keeps
low-level `unsafe` register access contained to a small, clearly-named
module instead of spread throughout the crate.

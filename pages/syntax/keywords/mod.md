---
title: "mod"
kind: keyword
embedded_support: full
groups: ["Modules, Crates & Visibility"]
related_concepts: [Modules]
related_syntax: [use, pub, crate, super, "::"]
see_also: [use, pub]
---

## Explanation

`mod` declares a module in one of two forms.

Written with a body — `mod name { ... }` — the module's contents sit
inline, right there in the braces, in the same file as the declaration.

Written as a bare declaration ending in a semicolon — `mod name;` — with
no body at all, it tells the compiler to load that module's contents from
another file instead. The compiler looks for one of two files, relative
to the directory of the file containing the `mod name;` line: `name.rs`,
or `name/mod.rs` (an older convention, still supported, and still the
only option once `name` itself needs to declare further submodules living
in a `name/` directory). Both forms declare exactly the same kind of
module to the compiler; the only difference is where its contents are
written down.

A module declared with either form can be prefixed with `pub` (see
[`pub`](pub.md)) to control its visibility, and can nest further `mod`
declarations, inline or file-loaded, in any combination. See
[Modules](../../concepts/modules-crates-visibility/modules.md) for the
mental model — why modules exist, how the tree maps to a public API — and
for the file-lookup rule in fuller depth; this page covers the grammar of
writing `mod` itself.

## Basic usage example

```
mod shapes {                            // <- `mod` opens an inline module
    pub struct Circle { pub radius: f64 }

    pub fn area(circle: &Circle) -> f64 {
        std::f64::consts::PI * circle.radius * circle.radius
    }
}

let unit_circle = shapes::Circle { radius: 1.0 };
println!("{}", shapes::area(&unit_circle));
```

## Best practices & deeper information

### Scenario: Designing a public API

A small metrics library keeps its aggregation and reporting logic in
separate files, loaded with `mod name;`, while re-exporting a short,
curated list of public names from the crate root.

```
// src/lib.rs
mod aggregator;               // <- `mod name;`: loads aggregator.rs as a private module
mod exporter;                 // <- `mod name;`: loads exporter.rs as a private module

pub use aggregator::Counter;  // curated public surface, independent of the file layout
pub use exporter::ConsoleExporter;

// src/aggregator.rs
pub struct Counter {
    count: u64,
}

impl Counter {
    pub fn new() -> Self {
        Counter { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}

// src/exporter.rs
pub struct ConsoleExporter;

impl ConsoleExporter {
    pub fn report(&self, count: u64) {
        println!("count = {count}");
    }
}
```

**Why this way:** the two `mod name;` declarations keep `aggregator` and
`exporter` as separate files mirroring the module tree on disk, while the
`pub use` re-exports mean callers depend on `crate::Counter` and
`crate::ConsoleExporter` rather than on knowing the internal file layout —
the curation the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends for a stable public surface.

### Scenario: Testing

A unit test module uses the *inline* form of `mod` — `mod name { ... }` —
rather than a separate file, since the tests live right alongside the code
they exercise.

```
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

#[cfg(test)]
mod tests {                     // <- `mod` here is inline (`mod name { ... }`), not file-loaded
    use super::*;

    #[test]
    fn converts_freezing_point() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
    }
}
```

**Why this way:** an inline `mod tests { ... }` compiled only under
`#[cfg(test)]` keeps the tests next to the code they cover instead of in a
separate file, the layout the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
uses throughout.

## Embedded Rust Notes

**Full support.** `mod` is a purely compile-time organizational construct
with no runtime representation, so both forms work identically in a
`#![no_std]` crate — there's no allocator or OS dependency either way.

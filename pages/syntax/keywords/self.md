---
title: "self"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism", "Modules, Crates & Visibility"]
related_concepts: [Ownership, Traits, "Borrowing (shared references)", "Mutable borrowing"]
related_syntax: [Self, trait, impl, mut, "&"]
see_also: [Self, mut, "&"]
---

## Explanation

`self` (lowercase) has two unrelated meanings, separated entirely by
position — the same by-position-disambiguation as `&`/`*`.

**1. The method receiver** (inside a `trait`/`impl` method signature).
`self` as the first parameter of a method means "the instance the method
was called on," and the exact form chosen tells the caller what happens
to their value:

- **`self`** (by value) — the method takes ownership of the receiver,
  consuming it. Once called, the original binding is gone, same as
  passing any owned value into a function. Common for builder-style
  chained methods and conversions that transform a value into something
  else.
- **`&self`** — the method borrows the receiver immutably. The most
  common receiver form: read-only inspection, caller keeps the value
  afterward.
- **`&mut self`** — the method borrows the receiver mutably, able to
  change it in place, without taking ownership.

`self` with no type annotation is shorthand for `self: Self` (or
`self: &Self` / `self: &mut Self`) — see [`Self`](self-type.md) for the
capital-`S` placeholder this refers to. A method with none of these as
its first parameter is an **associated function**, not a method
(`Client::new(...)`, not `client.new(...)`) — it's called on the type
itself, with no receiver at all.

**2. `self` in a path or `use` statement** — an entirely different sense,
referring to the *current module itself*, not an instance of anything.
Inside `use std::io::{self, Read};`, the `self` imports the `io` module
name itself (so `io::Error` is reachable), while `Read` alongside it
imports that one item directly — one `use` line covering both the module
and specific items from it, instead of two separate `use` statements.
`self` also appears in relative paths outside `use` (`self::helper()`
from within a module, meaning "look in this same module"), though `use
{self, ...}` grouped-import is by far the more common sighting.

## Usage examples

### Reading a value through `&self`

```
struct Order { total_cents: u64 }

impl Order {
    fn total(&self) -> u64 { // <- `&self`: reads the receiver, caller keeps ownership
        self.total_cents
    }
}
```

### Implementing traits

A builder type's chained methods take `self` by value, consuming and
returning the modified builder each step, while a plain inspection method
takes `&self` — the receiver form is chosen per method, based on what
that method actually does.

```
struct RequestBuilder {
    url: String,
    retries: u32,
}

impl RequestBuilder {
    fn retries(mut self, n: u32) -> Self { // <- `self` by value: consumes and returns the builder
        self.retries = n;
        self
    }

    fn url(&self) -> &str { // <- `&self`: read-only, caller keeps using the builder afterward
        &self.url
    }
}

let builder = RequestBuilder { url: "https://api.example.com".into(), retries: 0 };
let builder = builder.retries(3); // <- consumes the old `builder` binding, returns a new one
println!("{}", builder.url());
```

An owned `self` receiver is what makes fluent method
chaining possible — each call consumes the previous builder state and
hands back a new one — while `&self` on `url` signals "just reading, you
still own this afterward"; the
[Rust Book's method-syntax chapter](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
covers exactly this choice among `self`, `&self`, and `&mut self`.

### Designing a public API

Importing both a module and specific items from it in one line uses the
path-position sense of `self` — a distinct meaning from the receiver, but
still governed by its position (inside a `use` group, not a method
signature).

```
use std::io::{self, Read, Write}; // <- `self` here imports the `io` module itself, alongside two items from it

fn describe_error(err: &io::Error) -> String { // <- reachable because `self` imported the `io` module name
    format!("io error: {err}")
}

fn copy_all(mut source: impl Read, mut sink: impl Write) -> io::Result<u64> {
    std::io::copy(&mut source, &mut sink)
}
```

Without the `self` in the group, `io::Error` and
`io::Result` would be unreachable unless the module was imported
separately (`use std::io;` on its own line) — grouping `self` alongside
specific items is the idiomatic way to get both without two `use`
statements, as shown throughout the
[Rust Reference's use-declarations section](https://doc.rust-lang.org/reference/items/use-declarations.html).

## Embedded Rust Notes

**Full support** for both senses. Method receivers (`self`/`&self`/
`&mut self`) are core-language and central to `embedded-hal` driver APIs;
the module-path sense of `self` in `use` statements is likewise pure
compile-time name resolution, with no `std` dependency either way.

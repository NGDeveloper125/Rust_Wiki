---
title: "Self"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism"]
related_concepts: [Traits, "Associated types"]
related_syntax: [trait, impl, self]
see_also: [trait, impl, self]
---

## Explanation

`Self` (capital S) is a placeholder that stands for "the type this
`impl` or `trait` block is currently for." Inside `impl Client { ... }`,
`Self` means `Client`; inside `impl Greet for Cat { ... }`, `Self` means
`Cat`. It's used anywhere the concrete type name would otherwise be
repeated: constructor return types (`fn new() -> Self`), associated-type
assignments (`type Output = Self;`), and as a constructor path itself
(`Self { field: value }`, `Self::default()`).

`Self` differs from writing the concrete type name out explicitly in one
important way: it stays correct wherever the surrounding `impl`/`trait`
block itself is generic or reused, rather than being tied to one type
name that has to be kept in sync by hand.

- **Inside a generic `impl` block**, `Self` automatically means the
  specific instantiation, without repeating the type parameters:
  `impl<T> Wrapper<T> { fn new(value: T) -> Self { Wrapper { value } } }`
  — `Self` here means `Wrapper<T>`, saving you from writing
  `-> Wrapper<T>` and keeping it in sync if the struct is later renamed.
- **Inside a `trait`'s default method or signature**, `Self` means
  "whatever concrete type ends up implementing this trait" — a different
  type for every implementer, resolved fresh each time the trait is
  implemented. A `trait Greet { fn make() -> Self; }` requires each
  implementer's `make` to return *that implementer's own type*, not some
  fixed type chosen when the trait was written.

Renaming the type later, or implementing the same trait for a second
type, requires touching nothing inside a body that used `Self` — the
placeholder resolves fresh every time, whereas a hardcoded type name
would need to be found and updated at every occurrence.

## Usage examples

### Using `Self` as a constructor's return type

```
struct Client { host: String }

impl Client {
    fn new(host: &str) -> Self { // <- `Self` means `Client` here, without repeating the name
        Client { host: host.into() }
    }
}
```

### Creating a new object

A constructor's return type is conventionally written `Self`, not the
type's own name repeated — the two are identical here, but only one of
them survives a rename with zero edits.

```
pub struct Configuration {
    max_connections: u32,
}

impl Configuration {
    pub fn new(max_connections: u32) -> Self { // <- `Self`, not `Configuration`, per convention
        Self { max_connections } // <- `Self` also usable as the constructor path itself
    }
}

let config = Configuration::new(10);
```

The
[API Guidelines' C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#constructors-are-static-inherent-methods-c-ctor)
examples consistently use `Self` for a constructor's return type — if
`Configuration` is ever renamed, every `Self`-typed signature and
`Self { ... }` literal keeps compiling unchanged.

### Designing a public API

A trait method that must return "whatever type is implementing this
trait" can only be expressed with `Self` — writing a concrete type name
here would be wrong for every implementer except one.

```
trait Buildable {
    fn empty() -> Self; // <- `Self`: each implementer returns its own type, not a fixed one
}

struct Playlist { tracks: Vec<String> }
impl Buildable for Playlist {
    fn empty() -> Self { // <- resolves to `Playlist` for this impl specifically
        Playlist { tracks: Vec::new() }
    }
}

struct Queue { items: Vec<u32> }
impl Buildable for Queue {
    fn empty() -> Self { // <- the same trait method, resolving to `Queue` here instead
        Queue { items: Vec::new() }
    }
}
```

Writing `fn empty() -> Playlist` directly inside the
`trait Buildable` declaration wouldn't type-check for `Queue`'s
implementation at all — `Self` is the only way to express "return the
implementer's own type" in a trait signature, which is what lets a single
trait definition serve unrelated concrete types.

## Explanation (Embedded)

`Self` resolves exactly the same way under `#![no_std]` — a compile-time
placeholder with no runtime representation of its own. It shows up
constantly in HAL builder-style configuration types, where each setter
consumes and returns `Self` so a peripheral's configuration can be
chained fluently before the peripheral is finally initialized.

## Usage examples (Embedded)

### `Self` in a builder-style HAL config type

```
struct UartConfig {
    baud_rate: u32,
    parity: bool,
}

impl UartConfig {
    fn new() -> Self { // <- `Self` means `UartConfig` here
        Self { baud_rate: 9600, parity: false }
    }

    fn baud_rate(mut self, rate: u32) -> Self { // <- `Self`: consumes and returns the same config type
        self.baud_rate = rate;
        self
    }
}

let config = UartConfig::new().baud_rate(115_200);
```

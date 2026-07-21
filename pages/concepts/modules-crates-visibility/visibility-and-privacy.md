---
title: "Visibility & privacy (pub and friends)"
area: "Modules, Crates & Visibility"
embedded_support: full
groups: ["Modules, Crates & Visibility", "Structuring a Project", "Encapsulation", "Object-Oriented-ish Patterns", "Coming from Java / C#"]
related_syntax: [pub, crate, super]
see_also: ["Modules", "Crates"]
---

## Explanation

Visibility controls which code outside a given module can name or use an
item — a function, struct, field, enum variant, or module. By default,
every item in Rust is private: reachable only from the module that
defines it and that module's descendants. The `pub` keyword widens that;
scoped forms — `pub(crate)`, `pub(super)`, `pub(in some::path)` — widen
it to a specific ceiling instead of "everywhere," which lets an item be
shared with part of a crate without committing it to the crate's public
API.

The mental model is a boundary layered on top of the [module](modules.md)
tree: a child module can always see into its ancestors' private items —
a submodule automatically has access to everything its parent module
defines — but the reverse doesn't hold. A parent or sibling module can't
reach into a child module's private items unless the child explicitly
marks them `pub`. That asymmetry is what makes moving code into a nested
module a safe, additive way to hide detail: the module doing the hiding
never loses access to anything it already had.

The [crate](crates.md) boundary is the outer limit of this system:
`pub(crate)` is as visible as an item can be while still being
completely invisible to other crates, letting a crate share
implementation details across its own modules without those details
becoming part of what
[semver](dependency-management-and-semver.md) has to protect — plain
`pub` items are what a crate's version number actually promises to
callers.

Privacy is a design tool, not just an access switch. A struct with
private fields and a `pub` constructor (or no public constructor at all)
can guarantee its own invariants permanently, because no code outside
its module has any way to construct or mutate it into an invalid state —
there's no validation step for outside code to accidentally skip, since
outside code has no direct path to the fields at all.

This is Rust's version of encapsulation, and it plays a role similar to
`private`/`public`/`protected` in Java or C#, with two differences worth
noting: the boundary is the module, not the class, and there's no
`protected`-for-subclasses concept, because Rust has no inheritance for
it to apply to.

## Basic usage example

```
mod account {
    pub struct Account {
        id: u32,             // private: not reachable outside this module
        pub owner: String,   // <- pub: reachable from anywhere the module itself is
    }

    impl Account {
        pub fn new(id: u32, owner: String) -> Self {
            Account { id, owner }
        }
    }
}

fn main() {
    let acc = account::Account::new(1, "Priya".into());
    println!("{}", acc.owner);   // fine: owner is pub
    // println!("{}", acc.id);  // would fail to compile: id is private
}
```

## Best practices & deeper information

### Scenario: Designing a public API

An `Account`'s balance must never go negative, so the field stays private
and the only way to change it is through a method that enforces the
rule.

```
pub struct Account {
    owner: String,
    balance_cents: i64,   // <- private: no outside code can set this directly
}

impl Account {
    pub fn open(owner: String) -> Self {
        Account { owner, balance_cents: 0 }
    }

    pub fn deposit(&mut self, cents: i64) -> Result<(), &'static str> {
        if cents <= 0 {
            return Err("deposit must be positive");
        }
        self.balance_cents += cents;
        Ok(())
    }

    pub fn balance_cents(&self) -> i64 { // <- read-only access, no bypass of deposit's check
        self.balance_cents
    }
}
```

**Why this way:** keeping `balance_cents` private and reachable only
through `deposit` makes "the balance never goes negative" a guarantee of
the type itself rather than a rule callers have to remember to follow —
the
[API Guidelines' future-proofing chapter](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommends private fields for exactly this reason.

### Scenario: Testing

A crate's internal helper function needs to stay invisible to the
crate's users, but still be reachable from its own unit tests, which live
in a nested `tests` module in the same file.

```
pub fn format_receipt(total_cents: i64) -> String {
    format!("Total: {}", format_cents(total_cents))
}

fn format_cents(cents: i64) -> String { // <- private: an implementation detail, not part of the public API
    format!("${}.{:02}", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*; // <- test module is a descendant, so it can see `format_cents` despite it being private

    #[test]
    fn formats_dollars_and_cents() {
        assert_eq!(format_cents(1999), "$19.99");
    }
}
```

**Why this way:** because descendant modules can always see their
ancestors' private items, `#[cfg(test)] mod tests` reaches internal code
without anything being made `pub` just to satisfy a test — the pattern
the
[Rust Book's testing chapter](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
relies on throughout.

### Scenario: Documenting an API

rustdoc only generates public documentation pages for items reachable
from the crate's public API, so marking something private is also how
it stays out of the docs a crate's users see.

```
pub mod client {                     // <- pub: this module and its public items appear in rustdoc
    pub struct WeatherClient {
        api_key: String,             // <- private field: never shown in generated docs
    }

    impl WeatherClient {
        /// Creates a client authenticated with `api_key`.
        pub fn new(api_key: String) -> Self { // <- pub: shows up as public API documentation
            WeatherClient { api_key }
        }
    }

    fn sign_request(_key: &str) -> String { // <- private helper: absent from the published docs
        String::new()
    }
}
```

**Why this way:** rustdoc renders exactly the items visible from the
crate root outward, so a crate's visibility choices are effectively its
documentation's table of contents, per the
[rustdoc book](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html).

## Explanation (Embedded)

Visibility is enforced entirely at compile time and has no runtime
representation, so the mechanism is unchanged under `#![no_std]`. What's
genuinely important here is a convention, not a new rule: an embedded
HAL driver keeps every raw register read/write private and exposes only
a small set of `pub` methods that encode the hardware's actual safety
invariants (for example, "don't enable the peripheral before its clock
is enabled") — so `unsafe` register manipulation is contained to a
small, audited surface instead of scattered through application code.

## Basic usage example (Embedded)

```
pub struct Gpio {
    port: *mut u32,   // private: raw register pointer, never exposed
}

impl Gpio {
    pub fn set_high(&mut self, pin: u8) {
        unsafe { self.port.write_volatile(self.port.read_volatile() | (1 << pin)) } // <- unsafe write, contained here
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A GPIO driver's whole safety argument rests on nobody outside the module
ever getting a raw pointer to the peripheral's registers — the `pub`
surface only offers already-validated operations.

```
pub struct Gpio {
    port: *mut u32,        // <- private: the raw register address never leaves this module
}

impl Gpio {
    /// # Safety
    /// `port` must be a valid pointer to this peripheral's GPIO register block.
    pub unsafe fn new(port: *mut u32) -> Self {  // <- the one place the invariant is asserted, once
        Gpio { port }
    }

    pub fn set_high(&mut self, pin: u8) {         // <- safe: callers can no longer misuse the address
        unsafe {
            self.port.write_volatile(self.port.read_volatile() | (1 << pin));
        }
    }
}
```

**Why this way:** funneling every register access through a handful of
`pub` methods, instead of exposing the raw pointer, means the `unsafe`
contract only has to be checked once — at construction — rather than at
every call site, the "contain unsafety behind a safe API" practice the
[Rustonomicon](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
describes for wrapping raw-pointer/FFI-style access generally, applied
here to memory-mapped registers.

### Scenario: Documenting an API

A HAL crate's generated docs should read like a peripheral's user
manual, not its register map — so the register-level constants and raw
pointer types stay private and never appear in the published API.

```
mod registers {                 // <- private module: register offsets and bit positions
    pub(super) const ENABLE_BIT: u32 = 1 << 0;
}

pub struct Spi {
    // ...
}

impl Spi {
    /// Enables the SPI peripheral.
    pub fn enable(&mut self) {  // <- pub: this is what shows up in rustdoc
        // uses registers::ENABLE_BIT internally
    }
}
```

**Why this way:** keeping register offsets and bit masks in a
`pub(super)`-or-private `registers` module means rustdoc's generated
page for the crate shows only the peripheral-level operations users
actually call, per the
[rustdoc book](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)'s
"docs mirror the public API" behavior — applied here to keep a HAL's
documentation focused on hardware behavior, not register trivia.

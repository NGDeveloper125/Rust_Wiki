---
title: "#[deprecated]"
kind: attribute
embedded_support: full
groups: ["Lints & Diagnostics", "Design Patterns & Idioms"]
related_concepts: []
related_syntax: ["#[must_use]"]
see_also: []
---

## Explanation

`#[deprecated]` is placed above an item — a function, struct, enum,
trait, module, or any other item that can be named — and marks it as
deprecated: every use of that item from anywhere else in the crate, and
from any downstream crate that depends on it, produces a compiler
warning naming the deprecated item and pointing at the use site. The item
still compiles and still works exactly as before; `#[deprecated]` changes
nothing about behavior, only visibility of the warning that it's on its
way out.

Bare `#[deprecated]` produces a generic warning. Two optional fields
refine it, written as `name = value` pairs inside the parentheses and
combinable in one attribute:

- **`since = "..."`** — records the version the item became deprecated
  in, e.g. `since = "1.2.0"`, surfaced in the warning and in generated
  documentation so a reader can tell how long ago the deprecation
  happened.
- **`note = "..."`** — a human-readable message shown alongside the
  warning, almost always used to point at the replacement:
  `note = "use `new_fn` instead"`.

Both together — `#[deprecated(since = "1.2.0", note = "use `new_fn` instead")]`
— is the idiomatic full form: it tells a caller not just that something
is deprecated, but since when and what to use instead, right in the
compiler warning itself rather than requiring a trip to a changelog.

## Usage examples

### Deprecating a function in favor of its replacement

```
#[deprecated(since = "1.2.0", note = "use `parse_duration_ms` instead")] // <- warns at every call site
pub fn parse_duration(input: &str) -> u64 {
    parse_duration_ms(input)
}

pub fn parse_duration_ms(input: &str) -> u64 {
    input.trim_end_matches("ms").parse().unwrap_or(0)
}
```

### Designing a public API

A library replaces an older, ambiguously-named function with a clearer
one — rather than deleting the old function outright and breaking every
downstream caller in one release, it's kept working but marked deprecated
so callers get a compile-time nudge to migrate on their own schedule.

```
pub struct RetryPolicy {
    pub max_attempts: u32,
}

impl RetryPolicy {
    #[deprecated(since = "2.0.0", note = "use `RetryPolicy::with_max_attempts` instead")]
    pub fn new(max_attempts: u32) -> Self {
        RetryPolicy { max_attempts }
    }

    pub fn with_max_attempts(max_attempts: u32) -> Self {
        RetryPolicy { max_attempts }
    }
}

fn build_default_policy() -> RetryPolicy {
    RetryPolicy::with_max_attempts(3) // <- migrated call site: no warning
}
```

Deprecating first and removing later, rather than
removing immediately, gives downstream crates a compiler-enforced,
visible warning to act on across their own release cycle instead of a
sudden hard break — the
[Rust API Guidelines' predictability guidance](https://rust-lang.github.io/api-guidelines/predictability.html)
and standard semver practice both treat "deprecate, then remove in a
later breaking release" as the normal migration path for a stable public
API, and naming the replacement directly in `note` is what makes the
warning actionable rather than just a red flag.

## Explanation (Embedded)

`#[deprecated]` is a pure compile-time diagnostic mechanism with zero
runtime footprint, so it behaves identically under `#![no_std]` as
anywhere else — `core` and `alloc` items are deprecated using the exact
same attribute as `std` ones. It's a particularly natural fit for HAL
(hardware abstraction layer) crates, which routinely redesign their
peripheral APIs across major versions: an old register-access method gets
deprecated in favor of a newer, safer, or more ergonomic replacement, and
every downstream firmware crate calling the old method gets a compiler
warning naming the replacement — without an immediate breaking removal
that would force every consumer to migrate on the HAL maintainer's
schedule rather than their own.

## Usage examples (Embedded)

### Deprecating an old register-access method in a HAL crate

```
pub struct Gpio {
    // ...
}

impl Gpio {
    #[deprecated(since = "0.5.0", note = "use `Gpio::set_pin_state` instead; this method silently ignores an invalid pin index")] // <-
    pub fn set_pin(&mut self, pin: u8, high: bool) {
        self.set_pin_state(pin, high).ok();
    }

    pub fn set_pin_state(&mut self, pin: u8, high: bool) -> Result<(), &'static str> {
        if pin > 15 {
            return Err("pin index out of range for this GPIO port");
        }
        // ... write to the port's ODR register here
        Ok(())
    }
}
```

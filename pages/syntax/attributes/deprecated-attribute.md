---
title: "#[deprecated]"
kind: attribute
embedded_support: full
groups: ["Design Patterns & Idioms"]
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

## Basic usage example

```
#[deprecated(since = "1.2.0", note = "use `parse_duration_ms` instead")] // <- warns at every call site
pub fn parse_duration(input: &str) -> u64 {
    parse_duration_ms(input)
}

pub fn parse_duration_ms(input: &str) -> u64 {
    input.trim_end_matches("ms").parse().unwrap_or(0)
}
```

## Best practices & deeper information

### Scenario: Designing a public API

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

**Why this way:** deprecating first and removing later, rather than
removing immediately, gives downstream crates a compiler-enforced,
visible warning to act on across their own release cycle instead of a
sudden hard break — the
[Rust API Guidelines' predictability guidance](https://rust-lang.github.io/api-guidelines/predictability.html)
and standard semver practice both treat "deprecate, then remove in a
later breaking release" as the normal migration path for a stable public
API, and naming the replacement directly in `note` is what makes the
warning actionable rather than just a red flag.

## Embedded Rust Notes

**Full support.** `#[deprecated]` is a pure compile-time, zero-runtime-cost
diagnostic mechanism — it works identically in `#![no_std]` crates, and is
just as useful there for a HAL crate migrating callers from an old
peripheral API to a redesigned one across major versions.

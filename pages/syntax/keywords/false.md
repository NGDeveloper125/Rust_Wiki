---
title: "false"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [true]
see_also: [true]
---

## Explanation

`false` is the boolean literal for a false value, of type `bool`. Like
`true`, it is both a reserved keyword and a complete literal expression.

```
let done: bool = false;
```

See [`true`](true.md) for the surrounding notes on `bool` as a distinct,
non-numeric type with no implicit conversions.

## Basic usage example

```
let done: bool = false; // <- `false` is the boolean literal for a false value
```

## Best practices & deeper information

### Scenario: Validating input

A struct of feature flags implements `Default` so any new, unreleased
feature starts disabled unless explicitly opted into — `false` is the
conservative default for a flag that shouldn't be on until someone turns
it on.

```
struct FeatureFlags {
    enable_beta_ui: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        FeatureFlags {
            enable_beta_ui: false, // <- unreleased features default to `false` until explicitly opted in
        }
    }
}
```

**Why this way:** defaulting risky or unfinished behavior to `false`
means a missing configuration value fails closed rather than open, which
the
[API Guidelines' trait-implementation conventions](https://rust-lang.github.io/api-guidelines/predictability.html)
support by recommending types eagerly implement `Default` with a safe
baseline. See [`true`](true.md) for the fuller treatment of `bool`
literals.

## Embedded Rust Notes

**Full support.** Same as [`true`](true.md) — a `core` primitive, no `std`
dependency.

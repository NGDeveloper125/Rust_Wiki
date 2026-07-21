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

See [`true`](true.md) for the surrounding notes on `bool` as a distinct,
non-numeric type with no implicit conversions.

## Usage examples

### The `false` boolean literal

```
let done: bool = false; // <- `false` is the boolean literal for a false value
```

### Validating input

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

Defaulting risky or unfinished behavior to `false`
means a missing configuration value fails closed rather than open. Making
that the `Default` also lines up with the API Guidelines'
[C-COMMON-TRAITS](https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits),
which recommends types eagerly implement `Default`. See [`true`](true.md)
for the fuller treatment of `bool` literals.

## Explanation (Embedded)

As with [`true`](true.md), there's no genuine embedded-specific angle to
`false` — it's the same `core` primitive literal, identical under
`#![no_std]`. It's worth naming plainly rather than manufacturing a
distinction: the only embedded-flavored thing to say is what a `false`
typically represents in firmware — an interrupt flag that hasn't fired
yet, or a peripheral not yet ready.

## Usage examples (Embedded)

### A completion flag that starts false until an interrupt fires

```
let mut dma_transfer_complete = false; // <- `false` is the initial state before the DMA interrupt sets it

fn on_dma_complete(flag: &mut bool) {
    *flag = true;
}
```

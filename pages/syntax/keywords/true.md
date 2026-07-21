---
title: "true"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [false]
see_also: [false]
---

## Explanation

`true` is the boolean literal for a true value, of type `bool`. It is
simultaneously a keyword (reserved, cannot be used as an identifier) and
a literal expression — the only value of its kind is itself, unlike
numeric literals which have many possible values.

`bool` in Rust is a distinct one-byte type, not an alias for an integer —
there's no implicit conversion between `bool` and `i32`/`u8`/etc. in
either direction (an explicit `as` cast is required: `true as i32 == 1`).

## Usage examples

### The `true` boolean literal

```
let done: bool = true; // <- `true` is the boolean literal for a true value
```

### Validating input

A feature flag is naturally represented as a plain `bool`, opted into
explicitly with the `true` literal rather than an integer or string
sentinel.

```
struct FeatureFlags {
    enable_new_dashboard: bool,
}

let flags = FeatureFlags { enable_new_dashboard: true }; // <- `true` opts this flag in explicitly

if flags.enable_new_dashboard {
    // render the new dashboard
}
```

`bool` is a distinct one-byte type with no implicit
conversion to or from integers, per the
[`bool` primitive docs](https://doc.rust-lang.org/std/primitive.bool.html)
— using `true`/`false` for on/off state avoids the "is 1 true or is
non-zero true?" ambiguity that languages without a real boolean type have.

### Branching on data (pattern matching)

A single bare `bool` reads better with `if`/`else`, but matching several
booleans together as a tuple can express a small dispatch table more
directly than a nested `if`/`else` pyramid.

```
fn shipping_label(is_express: bool, is_international: bool) -> &'static str {
    match (is_express, is_international) {
        (true, true) => "express international", // <- matching `true`/`false` combinations directly
        (true, false) => "express domestic",
        (false, true) => "standard international",
        (false, false) => "standard domestic",
    }
}
```

Clippy's
[`match_bool`](https://rust-lang.github.io/rust-clippy/master/#match_bool)
lint flags matching on a single lone `bool` in favor of `if`/`else`, but
that lint doesn't apply once several booleans are matched together — the
tuple match above is the case where `match` genuinely reads better.

## Explanation (Embedded)

There's no genuine embedded-specific angle to `true` itself — it's a
`core` primitive literal, identical in representation and behavior under
`#![no_std]`, and nothing about the Classic/Embedded toggle changes here.
The only thing worth naming is what kind of `bool` tends to show up in
embedded code: a GPIO pin's output level, or a "ready"/"done" flag read
back from a peripheral, is `true`/`false` exactly like any other boolean.

## Usage examples (Embedded)

### Setting a GPIO output level

```
led.set_state(true); // <- `true` drives the pin to its "high" output level
```

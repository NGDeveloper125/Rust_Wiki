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

```
let done: bool = true;
```

`bool` in Rust is a distinct one-byte type, not an alias for an integer —
there's no implicit conversion between `bool` and `i32`/`u8`/etc. in
either direction (an explicit `as` cast is required: `true as i32 == 1`).

## Basic usage example

```
let done: bool = true; // <- `true` is the boolean literal for a true value
```

## Embedded Rust Notes

**Full support.** `bool` is a primitive type defined in `core`, not `std`
— identical representation and behavior on embedded targets.

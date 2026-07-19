---
title: "="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Ownership, "Move semantics"]
related_syntax: [let, "mut"]
see_also: [let]
---

## Explanation

`=` assigns a new value to an existing, mutable binding (or place
expression), and also separates a binding from its initializer in `let`:

```
let mut x = 5; // = here: initializer, not reassignment
x = 6;         // = here: reassignment, requires `mut`
```

`=` is not overloadable — assignment always has the same built-in
meaning: move (or copy, for `Copy` types) the right-hand value into the
left-hand place. Assigning to a place that holds a non-`Copy` value drops
the old value first. `=` is not an expression (it evaluates to `()`, and
using its result is discouraged/rare), unlike C where `a = b` returns the
assigned value and chained assignment (`a = b = c`) is idiomatic — that
pattern is unusual in Rust.

`=` also appears in generic-parameter defaults (`struct S<T = i32>`) and
associated-type bindings (`Item = T`), both unrelated to runtime
assignment.

## Embedded Rust Notes

**Full support.** Assignment and move semantics are core language
behavior — no `std` dependency.

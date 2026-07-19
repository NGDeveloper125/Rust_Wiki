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

## Basic usage example

```
let mut count = 0;
count = 5; // <- `=` assigns 5 to `count`
```

**Restriction:** reassigning with `=` requires the binding to be
declared `mut` — `let x = 0; x = 1;` without `mut` is a compile error.

## Best practices & deeper information

### Scenario: Modifying an existing object

Reassigning a mutable binding with `=` replaces its value wholesale,
which keeps the binding always representing one complete, valid state
rather than a half-updated one.

```
enum ConnectionState {
    Disconnected,
    Connected,
}

let mut state = ConnectionState::Disconnected;
// ... connection succeeds ...
state = ConnectionState::Connected; // <- `=` replaces the old value entirely, not piecemeal
```

**Why this way:** replacing the whole binding in one `=` avoids any
window where `state` is partially updated, echoing the "make invalid
states unrepresentable" idea from [Effective Rust](https://effective-rust.com/)
applied to a plain mutable variable rather than a struct's fields.

### Scenario: Creating a new object

The `=` in a `let` binds a new value to a new name; unlike a later
reassignment, this occurrence never requires `mut`.

```
struct Reading {
    sensor_id: u32,
    celsius: f64,
}

let reading = Reading { sensor_id: 7, celsius: 21.4 }; // <- `=` binds the new value to `reading`
```

**Why this way:** per the [Book's variables chapter](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html),
bindings are immutable by default — add `mut` only once a binding is
actually going to be reassigned later, keeping the default the more
restrictive, safer one.

## Embedded Rust Notes

**Full support.** Assignment and move semantics are core language
behavior — no `std` dependency.

---
title: "="
kind: operator
embedded_support: full
groups: [Assignment, Basics]
related_concepts: [Ownership, "Move semantics"]
related_syntax: [let, "mut"]
see_also: [let]
---

## Explanation

`=` assigns a new value to an existing, mutable binding (or place
expression), and also separates a binding from its initializer in `let` —
the same `=` token serves both roles, distinguished only by whether the
binding already existed.

`=` is not overloadable — assignment always has the same built-in
meaning: move (or copy, for `Copy` types) the right-hand value into the
left-hand place. Assigning to a place that holds a non-`Copy` value drops
the old value first. `=` *is* an expression, but it evaluates to `()`
rather than the assigned value — unlike C, where `a = b` returns the
value, so C-style chained assignment (`a = b = c`) doesn't type-check in
Rust (it would assign `()` to `a`).

`=` also appears in generic-parameter defaults (`struct S<T = i32>`) and
associated-type bindings (`Item = T`), both unrelated to runtime
assignment.

## Usage examples

### Reassigning a mutable binding

```
let mut count = 0;
count = 5; // <- `=` assigns 5 to `count`
```

**Restriction:** reassigning with `=` requires the binding to be
declared `mut` — `let x = 0; x = 1;` without `mut` is a compile error.

### Modifying an existing object

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

Replacing the whole binding in one `=` avoids any
window where `state` is partially updated, echoing the "make invalid
states unrepresentable" idea from [Effective Rust](https://effective-rust.com/)
applied to a plain mutable variable rather than a struct's fields.

### Creating a new object

The `=` in a `let` binds a new value to a new name; unlike a later
reassignment, this occurrence never requires `mut`.

```
struct Reading {
    sensor_id: u32,
    celsius: f64,
}

let reading = Reading { sensor_id: 7, celsius: 21.4 }; // <- `=` binds the new value to `reading`
```

Per the [Book's variables chapter](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html),
bindings are immutable by default — add `mut` only once a binding is
actually going to be reassigned later, keeping the default the more
restrictive, safer one.

## Explanation (Embedded)

`=` means the same thing under `#![no_std]` — plain assignment, move (or
copy) semantics, all core-language, nothing changes reaching a
microcontroller target. For most embedded code `=` is exactly as
ordinary as anywhere else: assigning into a local variable, a struct
field, or a `mut` binding inside a task. The one place worth calling out
is assigning into a `static mut` that's also touched from an interrupt
handler. The assignment itself is still just `=`, but reaching it at all
requires an `unsafe` block — `static mut` access requires `unsafe` even
for a plain write — and a bare `=` there is only sound if nothing else
can observe a half-written state, which is precisely the property a
genuinely concurrent interrupt can break. That's why real firmware
usually reaches for a `Cell`, `RefCell`, or atomic type wrapped in a
`static` instead of a bare `static mut`, using their own interior-mutability
methods rather than raw `=`, so the compiler enforces safety instead of
the programmer promising it via `unsafe`.

## Usage examples (Embedded)

### Assigning into a `static mut` shared with an interrupt handler

```
static mut TICK_COUNT: u32 = 0;

// Called only from the timer interrupt handler.
unsafe fn on_timer_tick() {
    TICK_COUNT = TICK_COUNT + 1; // <- `=` assigns the new value; `unsafe` is required just to touch a `static mut`
}
```

### The safer alternative: assigning through a `Cell`-wrapped static

```
use core::cell::Cell;

struct TickCounter(Cell<u32>);
unsafe impl Sync for TickCounter {} // sound only because access here is single-threaded (main + one interrupt)

static TICK_COUNT: TickCounter = TickCounter(Cell::new(0));

fn on_timer_tick() {
    TICK_COUNT.0.set(TICK_COUNT.0.get() + 1); // <- still an `=`-style replacement of the whole value, now via `Cell::set`
}
```

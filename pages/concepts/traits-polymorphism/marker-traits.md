---
title: "Marker traits (Send, Sync, Sized, Copy)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism"]
related_syntax: []
see_also: ["Send & Sync", "Copy vs Clone", "Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

A marker trait has no methods at all — implementing it adds no new
behavior to a type. Its entire purpose is to let the compiler (and other
trait bounds) know a type has some property, purely through the type
system:

- `Copy` marks a type as safe to duplicate with a simple bitwise copy
  (see [Copy vs Clone](../ownership-borrowing/copy-vs-clone.md)).
- `Sized` marks a type whose size is known at compile time — true for
  almost everything, which is why it's an implicit bound on generic
  parameters by default (opted out of with `?Sized`).
- `Send` marks a type as safe to move to another thread.
- `Sync` marks a type as safe to access from multiple threads at once
  through a shared reference (see [Send & Sync](../concurrency-async/send-and-sync.md)).

Most marker traits are auto-derived by the compiler for any type made
entirely of parts that already have the marker — a struct of `Send`
fields is automatically `Send` itself, with no `impl` needed. This is
what lets the compiler enforce thread-safety rules across an entire
program's type graph without every single type needing an explicit
annotation; the property propagates structurally, and only types doing
something genuinely unusual (raw pointers, certain FFI types) need to
opt out explicitly.

## Basic usage example

```
struct Sensor { reading: i32 } // <- auto-`Send`: every field (i32) is Send, so no impl is needed

fn assert_send<T: Send>() {}
assert_send::<Sensor>();
```

## Best practices & deeper information

### Scenario: Multi-threading

A type shared across threads only compiles where the API requires it if
every field is itself `Send`/`Sync` — the marker traits are what the
compiler checks, not something the code has to assert by hand.

```
struct SensorReading { value: f64, timestamp: u64 } // <- auto-Send/Sync: every field is

fn spawn_worker(reading: SensorReading) {
    std::thread::spawn(move || { // <- requires SensorReading: Send, checked at compile time
        println!("{}: {}", reading.timestamp, reading.value);
    });
}

spawn_worker(SensorReading { value: 21.5, timestamp: 1_700_000_000 });
```

**Why this way:** because `Send`/`Sync` propagate structurally from a
type's fields, most types are thread-safe to move or share automatically
— the
[API Guidelines' C-SEND-SYNC](https://rust-lang.github.io/api-guidelines/interoperability.html)
treats "is `Send`/`Sync` where possible" as a checklist item precisely
because losing it (e.g. by adding an `Rc` or raw pointer field) is easy to
do by accident.

### Scenario: Designing a public API

Manually implementing `Send`/`Sync` is `unsafe` and rare — it's only
needed when a type contains something that isn't automatically
thread-safe (a raw pointer, an FFI handle) and the author can prove, by
hand, that sharing it really is safe.

```
struct FfiHandle(*mut u8); // raw pointer: not auto-Send

// SAFETY: this handle is never dereferenced by more than one thread at a
// time, which the caller must uphold for this impl to stay sound.
unsafe impl Send for FfiHandle {} // <- manual, unsafe: author is vouching for thread-safety

fn move_to_worker(handle: FfiHandle) {
    std::thread::spawn(move || drop(handle)); // only compiles because of the impl above
}
```

**Why this way:**
[`std::marker::Send`](https://doc.rust-lang.org/std/marker/trait.Send.html)
is an `auto trait` — the compiler derives it automatically for ordinary
types, so a hand-written `unsafe impl` should be rare and deliberate,
reserved for the specific fields (raw pointers, FFI types) that opt a
type out of the automatic derivation.

## Embedded Rust Notes

**Full support.** All defined in `core` — no `std` dependency. `Send`/`Sync`
are especially relevant in embedded code sharing state between a main
loop and an interrupt handler, or between tasks in an async executor like
`embassy`.

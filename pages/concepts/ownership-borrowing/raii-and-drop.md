---
title: "RAII & the Drop trait"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Systems / Low-Level Programming", "Unique to Rust", "Coming from C / C++"]
related_syntax: []
see_also: ["Ownership", "Memory safety without a garbage collector"]
---

## Explanation

RAII — Resource Acquisition Is Initialization — ties a resource's
lifetime to a value's scope: acquiring the resource happens in a
constructor, releasing it happens automatically when the value is
dropped. Rust inherits this idea directly from C++, and builds it into
the language as the default way *any* resource — heap memory, a file
handle, a mutex lock, a network socket — gets cleaned up.

The mechanism is the `Drop` trait: implementing `fn drop(&mut self)` lets
a type run arbitrary cleanup code the instant its owner goes out of
scope, with no need for the programmer to remember to call it — the
compiler inserts the call automatically at every point a value's owner's
scope ends, including on early returns and (by default) during a panic's
unwinding.

Combined with ownership's single-owner guarantee, this is what lets Rust
promise deterministic, automatic cleanup without a garbage collector:
there's never ambiguity about *when* a value's resources should be
released, because there's never ambiguity about who owns it or when that
owner's scope ends. This is a stricter, more automatic version of the
same discipline C++ programmers already practice by hand with RAII guard
types — Rust just makes the compiler enforce that every type follows it,
rather than relying on the programmer to write correct destructors and
never forget to use them.

## Basic usage example

```
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        println!("cleaning up");
    }
}

{
    let _g = Guard;
    println!("using resource");
} // <- _g goes out of scope here: drop() runs automatically
```

## Embedded Rust Notes

**Full support** — and arguably more central to embedded Rust than to
hosted code. RAII is the idiomatic way embedded HAL crates model
peripheral ownership: a driver struct's `Drop` impl can disable a
peripheral, release a pin back to a default state, or turn off a clock
the instant it goes out of scope, with no OS process teardown to lean on
as a safety net the way a hosted program implicitly has.

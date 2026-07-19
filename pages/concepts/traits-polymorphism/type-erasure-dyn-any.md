---
title: "Type erasure (dyn Any & downcasting)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Type Erasure"]
related_syntax: []
see_also: ["Trait objects & dynamic dispatch (dyn Trait)"]
---

## Explanation

`dyn Any` is a special trait object that erases a value's concrete type
entirely, keeping only enough information to ask "is this actually a
`T`?" at runtime and, if so, recover a concrete reference to it —
downcasting:

```
fn describe(value: &dyn std::any::Any) {
    if let Some(n) = value.downcast_ref::<i32>() {
        println!("an i32: {n}");
    }
}
```

This is deliberately rare in idiomatic Rust — almost all polymorphism is
handled through ordinary trait objects or generics, where the compiler
knows and checks the relevant types statically. `Any` exists for the
genuine edge cases where a fully dynamic, type-unaware container is
unavoidable (a heterogeneous registry keyed by type, plugin systems that
hand back opaque values) — it trades the compiler's static guarantees for
a runtime check, which is why reaching for `Any` is usually a sign a
design could potentially be reworked around generics or an enum instead,
unless the dynamism is genuinely inherent to the problem.

## Embedded Rust Notes

**Full support.** `Any` and `downcast_ref` live in `core::any` — no
allocator needed for the reference-based form. As with ordinary trait
objects, only a `Box<dyn Any>` (an owned, heap-allocated erased value)
needs the `alloc` crate; borrowing-based downcasting does not.

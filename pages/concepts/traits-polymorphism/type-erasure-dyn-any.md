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

## Basic usage example

```
use std::any::Any;

fn describe(value: &dyn Any) {
    if let Some(n) = value.downcast_ref::<i32>() { // <- runtime check + recovered concrete type
        println!("an i32: {n}");
    }
}

describe(&42);
```

## Best practices & deeper information

### Scenario: Designing a public API

Reaching for `dyn Any` is usually a signal to step back and check whether
a generic function or an enum would express the same design without
giving up static type checking — but a genuinely dynamic case, like a
plugin registry keyed by type, is a legitimate use.

```
use std::any::{Any, TypeId};
use std::collections::HashMap;

struct PluginRegistry {
    plugins: HashMap<TypeId, Box<dyn Any>>, // <- narrow, deliberate use: keys aren't known until runtime
}

impl PluginRegistry {
    fn insert<T: Any>(&mut self, plugin: T) {
        self.plugins.insert(TypeId::of::<T>(), Box::new(plugin));
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.plugins.get(&TypeId::of::<T>())?.downcast_ref::<T>() // <- runtime check recovers the concrete type
    }
}
```

**Why this way:** every other trait/generic bound in this page's own
Explanation is checked at compile time; `Any` gives that up, so it earns
its place only where the set of types genuinely isn't known until runtime
— [`std::any::Any`](https://doc.rust-lang.org/std/any/trait.Any.html)
documents `TypeId`-keyed lookups like this as the intended use, not a
general-purpose substitute for generics or an enum.

## Embedded Rust Notes

**Full support.** `Any` and `downcast_ref` live in `core::any` — no
allocator needed for the reference-based form. As with ordinary trait
objects, only a `Box<dyn Any>` (an owned, heap-allocated erased value)
needs the `alloc` crate; borrowing-based downcasting does not.

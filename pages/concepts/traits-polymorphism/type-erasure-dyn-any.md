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
downcasting. For instance, a function taking `&dyn Any` can check
`value.downcast_ref::<i32>()` to test, at runtime, whether the erased
value is actually an `i32`, recovering a concrete reference if so.

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

**Restriction:** `Any` only covers `'static` types — `TypeId::of::<T>()`
requires `T: 'static`, so a type that borrows non-`'static` data can
never be erased to `dyn Any` or downcast back. The `T: Any` bounds above
already imply `'static`; a struct holding a `&'a str` would be rejected.

## Explanation (Embedded)

`Any` and `downcast_ref` live in `core::any`, so the reference-based form
works under `#![no_std]` with no allocator at all — only `Box<dyn Any>`
(an owned, heap-allocated erased value) needs `alloc`, same as any other
trait object.

Worth being honest about, though: type erasure via `Any` is genuinely
rarer in embedded code than in hosted Rust, and rarer than the already-low
bar set on the classic side of this page. Resource-constrained firmware
overwhelmingly prefers a fixed enum of known message/command kinds, or a
generic function bounded by a concrete trait, over anything that pays a
runtime `TypeId` check — both because the set of types involved is
usually genuinely fixed at compile time (there's no third-party plugin
loading a bare-metal binary), and because a `match` over an enum
compiles to a jump table the compiler can reason about, where `Any`
downcasting is an opaque runtime comparison the compiler can't optimize
across. Where `Any` does show up honestly is a debug/diagnostic console
task that needs to hold a small, fixed-capacity table of heterogeneous
handler types registered at startup — a genuinely dynamic-lookup need,
just a narrow one. Reaching for `Any` anywhere else in embedded code is
usually a sign an enum or a generic bound was available and simpler.

## Basic usage example (Embedded)

```
use core::any::Any;

fn describe(value: &dyn Any) {
    if let Some(n) = value.downcast_ref::<u16>() { // <- runtime check, no heap involved
        let _raw_adc_reading = n;
    }
}

describe(&512u16);
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A debug console task wants to store a handful of heterogeneous command
handlers registered at startup, looked up by type — a genuinely dynamic
case, but a narrow one; most embedded designs reach for a fixed enum of
known commands instead, and should, unless the handler set truly isn't
fixed at compile time.

```
use core::any::{Any, TypeId};

struct HandlerSlot {
    type_id: TypeId,
    handler: &'static dyn Any, // <- erased, but heap-free: a 'static reference, not Box<dyn Any>
}

struct HandlerTable {
    slots: [Option<HandlerSlot>; 4], // fixed capacity, no allocator needed
}

impl HandlerTable {
    fn get<T: Any>(&self) -> Option<&T> {
        self.slots.iter().flatten()
            .find(|slot| slot.type_id == TypeId::of::<T>())
            .and_then(|slot| slot.handler.downcast_ref::<T>()) // <- runtime check recovers the concrete type
    }
}
```

**Why this way:** a fixed-size array in place of a heap-allocated map
keeps the table allocator-free, which matters more here than in hosted
code — but the honest caveat still applies: this pattern earns its place
only where the handler set genuinely isn't known until runtime; a
compile-time-fixed set of commands is better served by an enum, which
[`std::any::Any`](https://doc.rust-lang.org/std/any/trait.Any.html)'s own
docs frame as the exception case, not the default.

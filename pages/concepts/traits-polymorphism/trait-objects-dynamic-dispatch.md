---
title: "Trait objects & dynamic dispatch (dyn Trait)"
area: "Traits & Polymorphism"
embedded_support: full
groups: ["Traits & Polymorphism", "Object-Oriented-ish Patterns", "Decoupling", "Polymorphism", "Type Erasure", "Coming from Java / C#"]
related_syntax: [dyn]
see_also: ["Static dispatch & monomorphization", "Traits", "On-stack dynamic dispatch"]
---

## Explanation

A trait object (`dyn Trait`) erases a value's concrete type, keeping only
that it implements a given trait — and resolves which implementation's
methods to call at runtime rather than compile time. Since its size isn't
known at compile time, a trait object almost always appears behind a
pointer: `&dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>`.

```
let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle), Box::new(Square)];
for s in &shapes {
    s.area(); // resolved at runtime via a vtable
}
```

This is what lets a single collection hold values of genuinely different
concrete types, as long as they all implement the same trait — something
[generics](../types-data-modeling/generics.md) alone can't do, since a
generic function/type is monomorphized into one specialized version *per*
concrete type, not a single version that handles several types
interchangeably at once.

The mechanism is a vtable: a small table of function pointers built for
each concrete type's implementation, attached alongside the data pointer
whenever a trait object is created. Calling a method through `dyn Trait`
means an extra indirection through that table — a small, real runtime
cost compared to [static dispatch](static-dispatch-monomorphization.md),
which is the tradeoff being made in exchange for the ability to mix
different concrete types behind one interface. This is Rust's nearest
analogue to interface-typed references in Java/C#, though notably
without null being a possible value.

## Basic usage example

```
trait Shape {
    fn area(&self) -> f64;
}
struct Circle;
impl Shape for Circle { fn area(&self) -> f64 { 3.14 } }
struct Square;
impl Shape for Square { fn area(&self) -> f64 { 4.0 } }

let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle), Box::new(Square)];
// <- `dyn Shape` erases the concrete type; each element can be a different one
for s in &shapes {
    println!("{}", s.area()); // resolved at runtime via a vtable
}
```

## Embedded Rust Notes

**Full support** for `&dyn Trait`/`&mut dyn Trait` — the vtable mechanism
itself needs no allocator, only a reference to existing data. `Box<dyn Trait>`
specifically needs the `alloc` crate and a configured global allocator;
where a heap isn't available, see
[on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)
for the allocator-free equivalent.

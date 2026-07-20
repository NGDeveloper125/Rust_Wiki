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
pointer: `&dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>`. For instance, a
`Vec<Box<dyn Shape>>` can hold `Circle`s and `Square`s side by side, and
calling `.area()` on each element resolves to the right implementation at
runtime via a vtable.

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

## Best practices & deeper information

### Scenario: Runtime polymorphism

An audio effect chain needs to apply an arbitrary, user-configured mix of
effects in sequence — the exact set and order aren't known until runtime,
which rules out a fixed enum or a single generic function.

```
trait Effect {
    fn apply(&self, sample: f32) -> f32;
}

struct Gain(f32);
impl Effect for Gain {
    fn apply(&self, sample: f32) -> f32 { sample * self.0 }
}

struct Clip(f32);
impl Effect for Clip {
    fn apply(&self, sample: f32) -> f32 { sample.clamp(-self.0, self.0) }
}

fn process(sample: f32, chain: &[Box<dyn Effect>]) -> f32 { // <- one signature, any mix of effect types
    chain.iter().fold(sample, |s, effect| effect.apply(s))
}

let chain: Vec<Box<dyn Effect>> = vec![Box::new(Gain(2.0)), Box::new(Clip(1.0))];
process(0.6, &chain);
```

**Why this way:** the chain's length and composition are decided at
runtime (config, user input), so no single generic instantiation could
cover it — this is exactly the heterogeneous-collection case the
[Rust Book](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
uses trait objects for.

### Scenario: Designing a public API

A plugin-style registry is a natural `Vec<Box<dyn Trait>>` API: the crate
owns the collection and dispatch logic, while callers register any type
implementing the trait.

```
trait Command {
    fn name(&self) -> &str;
    fn run(&self, input: &str);
}

pub struct Registry {
    commands: Vec<Box<dyn Command>>, // <- registry stores any Command impl behind one type
}

impl Registry {
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    pub fn dispatch(&self, name: &str, input: &str) {
        if let Some(cmd) = self.commands.iter().find(|c| c.name() == name) {
            cmd.run(input);
        }
    }
}
```

**Why this way:** designing `Command` to stay object-safe (no generic
methods, no returning `Self`) is what makes this API possible at all —
the [API Guidelines' C-OBJECT](https://rust-lang.github.io/api-guidelines/flexibility.html)
calls out object safety as a deliberate design goal for traits meant to be
used this way, and the shape matches the
[command pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html)
in the Rust Design Patterns book.

## Embedded Rust Notes

**Full support** for `&dyn Trait`/`&mut dyn Trait` — the vtable mechanism
itself needs no allocator, only a reference to existing data. `Box<dyn Trait>`
specifically needs the `alloc` crate and a configured global allocator;
where a heap isn't available, see
[on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)
for the allocator-free equivalent.

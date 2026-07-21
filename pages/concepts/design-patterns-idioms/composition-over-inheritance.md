---
title: "Composition over inheritance"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Object-Oriented-ish Patterns", "Decoupling", "Composition"]
related_syntax: [impl]
see_also: ["Compose structs", "Trait objects & dynamic dispatch (dyn Trait)", "Supertraits", "Default trait methods", "Structs"]
---

## Explanation

Rust has no struct inheritance: there is no `class Dog extends Animal`,
no base-class field layout a subtype automatically inherits, and no way
for one struct to "be" another. Code and data reuse instead happens
through **composition** — a struct holding another struct as a field —
and through **traits**, which share *behavior* (including
[default method bodies](../traits-polymorphism/default-trait-methods.md))
across otherwise unrelated types without requiring a common ancestor or
memory layout. Where an inheritance-based design asks "what is this a
kind of?", a Rust design asks "what is this made of, and what can it
do?" — composition answers the first with fields, traits answer the
second with `impl` blocks.

This isn't a workaround for a missing feature; it's a deliberate
response to problems inheritance creates. A deep class hierarchy tightly
couples every subclass to its ancestors' internal layout and behavior —
changing a base class can silently break subclasses built on top of it,
the classic "fragile base class" problem. Multiple inheritance (needing
to be *two* kinds of thing at once) forces awkward rules for which
ancestor's implementation wins on a name clash. Composition sidesteps
both: a struct's fields are exactly what's declared on it, a change to
one type's fields never reaches into another type's layout, and a
struct can hold as many component fields as it needs with no ambiguity
about which one supplies a given piece of behavior.

Where shared behavior genuinely is needed across otherwise-unrelated
types, a trait with default methods does what an inherited method would
— except a type opts in explicitly (`impl SharedTrait for MyType`)
rather than acquiring it automatically as a side effect of extending a
class, and a type can implement any number of such traits with no
multiple-inheritance ambiguity, since each trait's methods stay
namespaced to that trait. See [supertraits](../traits-polymorphism/supertraits.md)
for the closest Rust gets to "this trait requires that trait" — still a
composition of capabilities, not an is-a relationship.

One real anti-pattern worth naming so it isn't reached for by accident:
implementing `Deref`/`DerefMut` purely so a composed field's methods
become callable directly on the outer struct (`outer.inner_method()`
instead of `outer.inner.inner_method()`) reintroduces exactly the
coupling composition is meant to avoid, and the standard library
reserves `Deref` for genuine smart-pointer types. An explicit delegating
method costs a few extra lines and keeps the relationship honest. See
also [compose structs](compose-structs.md) for the data-shape half of
this idea — grouping *fields*, as opposed to this page's concern of
sharing *behavior*.

## Basic usage example

```
struct Health { hp: u32 }
impl Health {
    fn take_damage(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
    }
}

struct Inventory { items: Vec<String> }
impl Inventory {
    fn add(&mut self, item: &str) {
        self.items.push(item.to_string());
    }
}

struct Player { // <- composed of two independent components, not inherited from a base "Entity"
    health: Health,
    inventory: Inventory,
}

let mut player = Player {
    health: Health { hp: 100 },
    inventory: Inventory { items: Vec::new() },
};
player.health.take_damage(30);
player.inventory.add("sword");
println!("{} hp, {} items", player.health.hp, player.inventory.items.len());
```

## Best practices & deeper information

### Scenario: Designing a public API

An audio player needs to support several codecs and several output
devices; a class hierarchy (`Mp3FilePlayer extends FilePlayer`) would
force one class per codec/device combination, while composing a codec
and an output device as two independent fields lets any combination be
assembled without multiplying types.

```
trait Codec {
    fn decode(&self, data: &[u8]) -> Vec<i16>;
}

trait OutputDevice {
    fn play(&self, samples: &[i16]);
}

struct Mp3Codec;
impl Codec for Mp3Codec {
    fn decode(&self, data: &[u8]) -> Vec<i16> {
        data.iter().map(|&b| b as i16).collect()
    }
}

struct Speakers;
impl OutputDevice for Speakers {
    fn play(&self, samples: &[i16]) {
        println!("playing {} samples", samples.len());
    }
}

struct AudioPlayer<C: Codec, O: OutputDevice> { // <- composed of a codec and an output device, not inherited from a base player
    codec: C,
    output: O,
}

impl<C: Codec, O: OutputDevice> AudioPlayer<C, O> {
    fn play_file(&self, data: &[u8]) {
        let samples = self.codec.decode(data);
        self.output.play(&samples);
    }
}

let player = AudioPlayer { codec: Mp3Codec, output: Speakers };
player.play_file(&[1, 2, 3]);
```

**Why this way:** composing independent, trait-bounded components avoids
the combinatorial explosion of subclasses a codec/device hierarchy would
need — the
[Rust Book's chapter on object-oriented design](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
frames Rust's answer to "shared behavior across many types" as trait
implementations composed together, not inheritance.

### Scenario: Writing generic code

Logging an event needs to work for any type that can describe itself,
regardless of what it's internally composed of — a trait bound
expresses "can do X" without requiring a shared base type.

```
trait Describable {
    fn describe(&self) -> String;
}

struct Player { name: String, level: u32 }
impl Describable for Player {
    fn describe(&self) -> String {
        format!("{} (level {})", self.name, self.level)
    }
}

struct Enemy { kind: String }
impl Describable for Enemy {
    fn describe(&self) -> String {
        format!("a {}", self.kind)
    }
}

fn log_event<T: Describable>(item: &T) { // <- works for any type implementing Describable, no shared base type required
    println!("event: {}", item.describe());
}

log_event(&Player { name: "Nyx".into(), level: 4 });
log_event(&Enemy { kind: "goblin".into() });
```

**Why this way:** a generic bound over a shared trait gets the same
"operate uniformly over many types" benefit an inheritance hierarchy
would provide, without requiring `Player` and `Enemy` to share any base
type or field layout — the same trait-bound mechanism the
[Rust Book's generics chapter](https://doc.rust-lang.org/book/ch10-02-traits.html)
uses throughout.

## Embedded Rust Notes

**Full support.** Struct composition and trait default methods are
core-language mechanisms with zero runtime cost beyond ordinary field
layout and static or vtable dispatch, so both work identically under
`#![no_std]`. Neither implies heap allocation on its own; allocation only
enters if a composed component's own type happens to need it (a `Vec`
field, for instance), which is a property of that component, not of
composition itself.

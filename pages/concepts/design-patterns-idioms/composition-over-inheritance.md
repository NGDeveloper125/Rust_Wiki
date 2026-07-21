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

## Explanation (Embedded)

Composition over inheritance isn't just applicable to embedded Rust — the
entire `embedded-hal` ecosystem is built as a direct consequence of it
having no other option. There is no chip-specific base class a driver
could extend even if Rust had inheritance: a temperature sensor driver
written against `embedded-hal`'s `I2c` trait doesn't know or care whether
it's running against an STM32, an nRF52, or a Linux-hosted
`linux-embedded-hal` implementation talking to real hardware over
`/dev/i2c-1`. The driver is generic over the trait, composed with
whatever concrete HAL implementation the target board provides, and
every one of those HAL crates independently implements the same small
set of traits (`embedded_hal::i2c::I2c`, `embedded_hal::spi::SpiBus`,
`embedded_hal::digital::OutputPin`) without any of them needing to share
an ancestor. A driver written once against the trait works, unmodified,
on any chip whose HAL crate implements it — the payoff composition over
inheritance promises in the abstract is, in embedded Rust, the literal
reason a driver crate can be chip-agnostic at all.

The anti-pattern warning from the classic page — reaching for `Deref` to
fake inherited behavior — shows up in embedded code specifically around
HAL wrapper types (see [Anti-pattern: Deref
polymorphism](anti-pattern-deref-polymorphism.md)'s embedded notes for a
worked I2C example); the fix is the same explicit-delegation-or-shared-trait
answer, just with an `embedded-hal` trait usually already sitting there
as the shared behavior to compose against instead of a bespoke one.

## Basic usage example (Embedded)

```
trait TemperatureSensor {
    fn read_celsius(&mut self) -> f32;
}

struct Bme280<I2C> { // <- generic over any I2C implementation, composed rather than inherited
    i2c: I2C,
}

impl<I2C> TemperatureSensor for Bme280<I2C> {
    fn read_celsius(&mut self) -> f32 {
        21.5 // stands in for a real register read over `self.i2c`
    }
}

fn log_temperature(sensor: &mut dyn TemperatureSensor) {
    println!("{:.1} C", sensor.read_celsius());
}
```

## Best practices & deeper information (Embedded)

### Scenario: Designing a public API

A sensor driver crate needs to run on any board's I2C peripheral without
depending on a specific chip's HAL — composing the driver generically
over the `embedded-hal` `I2c` trait instead of any concrete
implementation is what makes that possible.

```
trait I2c {
    fn write_read(&mut self, addr: u8, out: &[u8], in_: &mut [u8]) -> Result<(), ()>;
}

struct Bme280<I2C: I2c> { // <- composed of any I2C implementation, not inherited from a chip-specific base
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c> Bme280<I2C> {
    fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    fn read_celsius(&mut self) -> Result<f32, ()> {
        let mut raw = [0u8; 2];
        self.i2c.write_read(self.address, &[0xFA], &mut raw)?;
        Ok(i16::from_be_bytes(raw) as f32 / 100.0)
    }
}

struct Stm32I2c1; // a board's concrete HAL type
impl I2c for Stm32I2c1 {
    fn write_read(&mut self, _addr: u8, _out: &[u8], in_: &mut [u8]) -> Result<(), ()> {
        in_.copy_from_slice(&[0x08, 0x34]); // stands in for a real bus transaction
        Ok(())
    }
}

let mut sensor = Bme280::new(Stm32I2c1, 0x76);
println!("{:.2} C", sensor.read_celsius().unwrap());
```

**Why this way:** `Bme280<I2C>` never names a concrete chip anywhere in
its own code, so the exact same driver crate compiles and runs unchanged
against any board whose HAL implements the `I2c` trait — this is the
`embedded-hal` ecosystem's core design point, and the
[embedded-hal project itself](https://github.com/rust-embedded/embedded-hal)
documents composing driver crates against its traits as the mechanism
that makes drivers portable across chip families with no shared base
type anywhere in the picture.

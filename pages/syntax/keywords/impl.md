---
title: "impl"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism"]
related_concepts: [Traits, "Static dispatch & monomorphization", "Trait objects & dynamic dispatch (dyn Trait)"]
related_syntax: [trait, dyn, Self]
see_also: [trait, dyn]
---

## Explanation

`impl` introduces a block of code attached to a type, and means one of two
unrelated things depending on whether a trait name follows it:

1. **`impl Trait for Type { ... }`** — implements a [`trait`](trait.md)
   for a concrete type, providing bodies for the trait's required methods
   (and optionally overriding its default ones). This is the only place a
   trait's methods get connected to a specific type.
2. **`impl Type { ... }`** — an **inherent impl block**, with no trait
   involved at all. Methods and associated functions defined here belong
   to the type itself, not to any trait — `Vec::new()`, `Vec::push`, and
   most of a type's everyday API are inherent, defined in a plain
   `impl Vec<T> { ... }` block in the standard library's own source.

A type can have any number of `impl` blocks — inherent ones and
trait-implementing ones — spread across a module or even merged for
organization; the compiler treats every `impl Type { ... }` block for the
same type as one contiguous set of inherent items, and likewise for each
distinct trait implementation.

`impl` also appears in a third, syntactically unrelated position:
**`impl Trait`** in argument or return-type position (`fn process(item: impl Display)`,
`fn make_iter() -> impl Iterator<Item = u32>`). This isn't an impl
*block* at all — no braces, no body — it's a type-position shorthand
meaning "some concrete type implementing this trait, chosen by the
caller (argument position) or the function (return position), but not
named." Don't confuse it with `impl Trait for Type`: same keyword,
completely different grammar position and meaning, the same
by-position-disambiguation pattern seen with tokens like [`&`](../operators/ampersand.md).

## Basic usage example

```
struct Cat;

impl Cat { // <- inherent impl block: no trait involved
    fn new() -> Self { Cat }
}

trait Greet {
    fn greet(&self) -> String;
}

impl Greet for Cat { // <- trait impl block: connects Greet's methods to Cat
    fn greet(&self) -> String { "meow".into() }
}
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

A plugin system needs many unrelated types behind one interface — each
type gets its own `impl Trait for Type` block, and callers hold them all
as `Box<dyn Trait>` without caring which concrete type is underneath.

```
trait Command {
    fn run(&self, input: &str);
}

struct EchoCommand;
impl Command for EchoCommand { // <- trait impl: connects Command's method to EchoCommand
    fn run(&self, input: &str) {
        println!("{input}");
    }
}

struct UppercaseCommand;
impl Command for UppercaseCommand { // <- a second, independent trait impl for a different type
    fn run(&self, input: &str) {
        println!("{}", input.to_uppercase());
    }
}

let commands: Vec<Box<dyn Command>> = vec![Box::new(EchoCommand), Box::new(UppercaseCommand)];
for command in &commands {
    command.run("status: ready");
}
```

**Why this way:** each `impl Command for ...` block is independent and can
live in its own module or crate, which is exactly the decoupling
[trait objects & dynamic dispatch](../../concepts/traits-polymorphism/trait-objects-dynamic-dispatch.md)
relies on — the trait, not a shared base type, is what ties these
implementations together.

### Scenario: Writing generic code

A function that only reads its argument through one trait's methods can
accept `impl Trait` in argument position instead of a generic type
parameter with a bound — same static dispatch, terser signature, at the
cost of the caller being unable to name the type explicitly.

```
fn log_reading(value: impl std::fmt::Display) { // <- `impl Trait` in argument position: not a block, no braces
    println!("reading: {value}");
}

log_reading(21.5);
log_reading("overheating");
```

**Why this way:** `impl Trait` in argument position desugars to the same
monomorphized generic function as `fn log_reading<T: Display>(value: T)`
— see
[static dispatch & monomorphization](../../concepts/traits-polymorphism/static-dispatch-monomorphization.md)
for why this costs nothing at runtime; reach for an explicit generic
parameter instead once the signature needs to refer to `T` more than
once (e.g. two parameters that must share the same concrete type).

## Embedded Rust Notes

**Full support.** Inherent and trait `impl` blocks are core-language,
allocator-free, and used identically in `#![no_std]` — `embedded-hal`
driver crates are built almost entirely out of `impl Trait for Type`
blocks connecting a vendor's peripheral type to the hardware-abstraction
traits.

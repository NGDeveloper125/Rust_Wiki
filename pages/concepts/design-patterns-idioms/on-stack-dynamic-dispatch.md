---
title: "On-stack dynamic dispatch"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Polymorphism"]
related_syntax: [dyn]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "The strategy pattern", "Static dispatch & monomorphization"]
---

## Explanation

A trait object almost always shows up behind `Box` in introductory
examples — `Box<dyn Trait>` — which quietly bundles a heap allocation
into every mention of dynamic dispatch. But the actual mechanism behind
[trait objects](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
is just a fat pointer: a data pointer plus a vtable pointer, and neither
half requires the data itself to live on the heap. `&dyn Trait` and
`&mut dyn Trait` build that same fat pointer around a plain reference to
a value that can live anywhere — most usefully, as an ordinary local on
the stack — which means runtime polymorphism and heap allocation are two
independent decisions, not one, even though beginner examples usually
reach for both at once.

The idiom is to keep concrete values as ordinary stack-local bindings
and reach for `&dyn Trait` only at the single point where one variable
needs to refer to "whichever one of these it turns out to be" —
typically choosing between two or more concrete implementors with an
`if` or `match`, then binding the result through one `&dyn Trait`-typed
variable that outlives the branch. No `Box::new` is involved anywhere:
nothing is heap-allocated, nothing needs a `Drop` to free it, and the
values are cleaned up exactly the way any other stack local would be, at
the end of their scope.

This matters most in two situations: a hot loop where the allocation
itself — not the vtable indirection — would be the actual performance
cost, and any `#![no_std]` environment with no allocator at all, where
`Box<dyn Trait>` simply isn't available but `&dyn Trait` works
unchanged. It's also the right call any time the concrete value's owner
already exists on the stack for the whole span of the call: boxing it
first would add an allocation purely to erase a type that was going to
be dropped at the very same point regardless.

## Basic usage example

```
trait Logger {
    fn log(&self, message: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("console: {message}");
    }
}

fn run(logger: &dyn Logger) { // <- takes a reference, never a Box: no heap allocation required
    logger.log("started");
}

let console = ConsoleLogger; // <- an ordinary stack value, not boxed
run(&console);
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

At startup, a CLI reads a `--verbose` flag and must pick one of two
logger implementations for the rest of the run, without allocating just
to hold that choice.

```
trait Logger {
    fn log(&self, message: &str);
}

struct ConsoleLogger;
impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("console: {message}");
    }
}

struct SilentLogger;
impl Logger for SilentLogger {
    fn log(&self, _message: &str) {}
}

fn configure_logger(verbose: bool) {
    let console = ConsoleLogger; // <- both candidates live on the stack, never boxed
    let silent = SilentLogger;

    let logger: &dyn Logger = if verbose { &console } else { &silent }; // <- one reference binds to either concrete stack value
    logger.log("configuration complete");
}

configure_logger(true);
```

**Why this way:** binding a single `&dyn Logger` to whichever concrete
stack value applies is exactly the technique the
[Rust Design Patterns' on-stack dynamic dispatch idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
names — the branch decides which concrete type is selected, but no
allocation happens anywhere in the process.

### Scenario: Boxing and heap allocation

Reaching for `Box<dyn Trait>` out of habit — when the concrete value
already lives on the stack for the entire call — adds an allocation that
buys nothing over a plain reference.

```
trait Shape {
    fn area(&self) -> f64;
}
struct Circle { radius: f64 }
impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

// AVOID: boxing a value that already lives on the stack for the whole call
fn print_area_boxed(shape: Box<dyn Shape>) {
    println!("{}", shape.area());
}
print_area_boxed(Box::new(Circle { radius: 2.0 })); // <- allocates just to erase the type for one call

// PREFER: a reference erases the type with no allocation at all
fn print_area(shape: &dyn Shape) {
    println!("{}", shape.area());
}
let circle = Circle { radius: 2.0 };
print_area(&circle); // <- no Box, no heap allocation, same runtime dispatch
```

**Why this way:** `Box` is for cases needing genuine ownership transfer
or a lifetime that outlives the current stack frame; when neither
applies, the
[Rust Design Patterns' on-stack dynamic dispatch idiom](https://rust-unofficial.github.io/patterns/idioms/on-stack-dyn-dispatch.html)
argues a plain `&dyn Trait` reference gets the same dispatch with none
of the allocation cost.

## Embedded Rust Notes

**Full support — this is the whole point of the idiom.** `&dyn
Trait`/`&mut dyn Trait` need only a reference to existing data, so they
work unchanged in `#![no_std]` with no allocator configured at all.
`Box<dyn Trait>` needs the `alloc` crate plus a `#[global_allocator]`;
on-stack dynamic dispatch is precisely the technique that gets the same
runtime polymorphism without either requirement, which is why it shows
up throughout embedded and other allocator-free Rust code.

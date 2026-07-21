---
title: "Smart pointers (Box<T>)"
area: "Ownership & Borrowing"
embedded_support: partial
groups: ["Ownership & Borrowing", "Boxing", "Sharing & Mutating Data Safely", "Coming from C / C++"]
related_syntax: []
see_also: ["Stack vs heap allocation", "Recursive types (via Box<T>)", "Deref & DerefMut coercion"]
---

## Explanation

`Box<T>` is the simplest smart pointer in Rust: it places a value on the
heap instead of the stack, while still behaving like a single, ordinary
owner of that value — moving a `Box` moves ownership of the heap
allocation, and when the `Box` is dropped, the heap memory is freed
immediately, with no reference counting or garbage collection involved.

It exists to solve two problems plain stack-allocated values can't: a
value whose exact size isn't known until runtime (a trait object,
`Box<dyn Trait>`) can't live directly on the stack, since the compiler
needs to know a type's size at compile time to allocate stack space for
it — putting it behind a `Box` gives it a fixed size (a single pointer)
regardless of what it points to. Similarly, a
[recursive type](../types-data-modeling/recursive-types-via-box.md) — a struct or enum that
contains itself — would need infinite size if stored inline, but a `Box`
breaks the cycle by storing a pointer instead of the value directly.

Unlike `Rc`/`Arc`, `Box<T>` has exactly one owner, same as any other
value — it changes *where* the data lives (heap instead of stack), not
*how many owners* it can have. This makes it the closest Rust analogue to
C++'s `std::unique_ptr`, and the one to reach for whenever heap allocation
is needed but shared ownership isn't.

## Basic usage example

```
let boxed: Box<i32> = Box::new(5); // <- value is allocated on the heap; boxed is its sole owner
println!("{boxed}");
```

## Best practices & deeper information

### Scenario: Boxing and heap allocation

An AST-like `Expr` type that contains itself needs `Box` to give the
recursive variant a fixed, known size — without it, the type would need
infinite space.

```
enum Expr {
    Literal(i32),
    Add(Box<Expr>, Box<Expr>), // <- Box breaks the infinite size: each variant is one pointer
}

fn eval(e: &Expr) -> i32 {
    match e {
        Expr::Literal(n) => *n,
        Expr::Add(lhs, rhs) => eval(lhs) + eval(rhs),
    }
}

let expr = Expr::Add(Box::new(Expr::Literal(2)), Box::new(Expr::Literal(3)));
println!("{}", eval(&expr));
```

**Why this way:** without `Box`, `Expr::Add(Expr, Expr)` would need
infinite size — an `Expr` containing two more `Expr`s, recursively.
Boxing the recursive fields gives the compiler a fixed-size pointer to
store instead, which the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) covers as
`Box`'s canonical use case for recursive types.

### Scenario: Runtime polymorphism

A plugin-style list of handlers with different concrete types needs
somewhere to live together despite not sharing a size — `Box<dyn Trait>`
erases the concrete type behind a uniform, heap-allocated pointer.

```
trait Handler {
    fn handle(&self, event: &str);
}

struct Logger;
impl Handler for Logger {
    fn handle(&self, event: &str) { println!("log: {event}"); }
}

struct Notifier;
impl Handler for Notifier {
    fn handle(&self, event: &str) { println!("notify: {event}"); }
}

let handlers: Vec<Box<dyn Handler>> = vec![Box::new(Logger), Box::new(Notifier)]; // <- heterogeneous, heap-allocated trait objects
for h in &handlers {
    h.handle("order.created");
}
```

**Why this way:** `Box<dyn Handler>` is needed because `Logger` and
`Notifier` have different sizes and the `Vec` needs one uniform element
type — boxing erases the concrete type behind a fixed-size pointer plus a
vtable, which the
[Rust Book](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)
recommends specifically for heterogeneous collections like this one,
reaching for `&dyn Handler` instead when a non-owning borrow will do.

## Explanation (Embedded)

`Box<T>` is defined in `alloc`, not `core`, so it requires `extern crate
alloc;` plus a configured `#[global_allocator]` — without that setup,
`Box::new(...)` doesn't compile under `#![no_std]` at all. The
[`box` keyword page](../../syntax/keywords/box.md) and the
[`vec!` macro page](../../syntax/macros/vec-macro.md) both cover this
`alloc`-vs-no-heap caveat in detail, including the reason there's no
direct `heapless` substitute for `Box` itself (a fixed-capacity collection
needs its element size known at compile time, while `Box`'s entire
purpose — most visibly `Box<dyn Trait>` — is holding something whose size
isn't known until runtime); this page won't repeat that mechanical
explanation.

What's specific to *this* page — the concept of reaching for a smart
pointer at all — is the design question embedded code has to ask before
boxing anything: does this genuinely need heap indirection, or does a
fixed-size alternative already work? A recursive type with an unbounded
depth (a general expression tree, a linked list of arbitrary length)
structurally needs some form of indirection, and if `alloc` is already in
the project for other reasons, `Box` is a perfectly reasonable way to get
it. But a lot of the shapes that reach for `Box` in hosted code — a
handful of known message variants, a small, fixed set of driver
implementations behind a trait — don't actually need heap allocation at
all: a plain (non-boxed) enum over the known variants, or `&dyn Trait`
for one concrete instance at a time (see
[on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)),
gets the same abstraction with a compile-time-known size and no allocator
requirement. The embedded-idiomatic default is to reach for the
no-allocation shape first and bring in `Box` only when the data
genuinely has no fixed upper bound.

## Basic usage example (Embedded)

```
extern crate alloc;
use alloc::boxed::Box;

let boxed: Box<i32> = Box::new(5); // <- identical to hosted Rust, once alloc + a #[global_allocator] exist
println!("{boxed}");
```

## Best practices & deeper information (Embedded)

### Scenario: Boxing and heap allocation

An AST-style recursive `Expr` type has no fixed maximum depth, so it
structurally needs indirection — once a project already depends on
`alloc`, boxing the recursive fields is the direct embedded equivalent of
the hosted version of this example.

```
extern crate alloc;
use alloc::boxed::Box;

enum Expr {
    Literal(i32),
    Add(Box<Expr>, Box<Expr>), // <- same fix as hosted Rust: Box breaks the infinite size
}

fn eval(e: &Expr) -> i32 {
    match e {
        Expr::Literal(n) => *n,
        Expr::Add(lhs, rhs) => eval(lhs) + eval(rhs),
    }
}
```

**Why this way:** an expression tree's depth depends on runtime input, so
no compile-time-fixed representation can bound it in general — `Box` is
the right tool exactly when the recursion is genuinely unbounded and
`alloc` is already available; if depth *is* bounded in practice (a
protocol that only ever nests a few levels deep), a fixed-size
representation avoids paying for an allocator at all, which is the
allocator-free tradeoff the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) frames as
`Box`'s recursive-type use case in the first place.

### Scenario: Runtime polymorphism

A small, fixed set of sensor drivers behind one trait doesn't need heap
allocation to be handled polymorphically — `&dyn Trait` gives one
concrete instance dynamic dispatch with a compile-time-known size,
without `Box<dyn Trait>`'s `alloc` requirement.

```
trait Sensor {
    fn read(&self) -> f32;
}

struct Thermometer;
impl Sensor for Thermometer {
    fn read(&self) -> f32 { 21.4 }
}

fn log_reading(sensor: &dyn Sensor) { // <- &dyn Trait: no heap, no alloc, works on a bare #![no_std] target
    println!("{}", sensor.read());
}

let thermometer = Thermometer;
log_reading(&thermometer);
```

**Why this way:** `&dyn Sensor` gets the same erased-type, dynamic-dispatch
behavior `Box<dyn Sensor>` would, without allocating, as long as the
caller can keep the concrete value alive on the stack for the borrow's
duration — the right choice whenever one instance is used at a time rather
than a heterogeneous collection genuinely needing owned storage; see
[on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)
for the fuller treatment of this substitute.

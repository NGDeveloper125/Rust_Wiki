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
[recursive type](recursive-types-via-box.md) — a struct or enum that
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

## Embedded Rust Notes

**Partial support.** `Box<T>` lives in `alloc`, not `core` — it needs
`extern crate alloc;` and a `#[global_allocator]`. Without one configured
(common in small, deterministic embedded projects), `Box` isn't
available at all; `heapless` types or plain stack allocation are the
usual allocator-free alternative. Where dynamic dispatch is still needed
without a heap, [on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)
(`&dyn Trait` instead of `Box<dyn Trait>`) is the idiomatic embedded
substitute.

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

## Embedded Rust Notes

**Partial support.** `Box<T>` lives in `alloc`, not `core` — it needs
`extern crate alloc;` and a `#[global_allocator]`. Without one configured
(common in small, deterministic embedded projects), `Box` isn't
available at all; `heapless` types or plain stack allocation are the
usual allocator-free alternative. Where dynamic dispatch is still needed
without a heap, [on-stack dynamic dispatch](../design-patterns-idioms/on-stack-dynamic-dispatch.md)
(`&dyn Trait` instead of `Box<dyn Trait>`) is the idiomatic embedded
substitute.

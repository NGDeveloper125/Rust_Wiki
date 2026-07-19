---
title: "Deref & DerefMut coercion"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing"]
related_syntax: ["*"]
see_also: ["Smart pointers (Box<T>)", "Shared ownership (Rc & Arc)"]
---

## Explanation

`Deref` and `DerefMut` let a smart pointer type transparently behave like
a reference to whatever it wraps. A type implementing `Deref<Target = T>`
can be used almost anywhere a `&T` is expected — most visibly, calling a
method defined on `T` directly on the smart pointer (`my_box.method()`
instead of `(*my_box).method()`), because the compiler automatically
inserts as many derefs as needed to find a matching method.

This coercion is what makes `Box<T>`, `Rc<T>`, `String` (which derefs to
`&str`), and `Vec<T>` (which derefs to `&[T]`) feel ergonomic to use day
to day — you rarely need to think about the wrapper layer at all, because
method calls, and reference coercions in function-argument position, see
straight through it to the underlying type.

The tradeoff worth knowing about: overusing custom `Deref` impls purely
to fake inheritance-like "is-a" relationships between unrelated types is
a recognized anti-pattern in Rust (sometimes called "Deref polymorphism")
— `Deref` is meant to model "acts like a reference to," not "is a kind
of," and stretching it to the latter tends to produce confusing method
resolution rather than genuinely reusable abstraction.

## Embedded Rust Notes

**Full support.** `Deref`/`DerefMut` live in `core::ops` — no allocator
dependency (the mechanism works the same whether or not the smart pointer
being deref'd happens to need `alloc`, e.g. it applies just as well to
allocator-free wrapper types).

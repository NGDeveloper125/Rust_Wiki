---
title: "Recursive types (via Box<T>)"
area: "Types & Data Modeling"
embedded_support: partial
groups: ["Types & Data Modeling", "Boxing", "Recursive Data Structures"]
related_syntax: []
see_also: ["Smart pointers (Box<T>)", "Enums (algebraic data types)"]
---

## Explanation

A type that contains itself directly — a linked-list node holding another
node, a tree node holding child nodes of the same type — can't be
represented as-is, because the compiler needs to know a type's exact size
at compile time, and a type containing itself would have to be infinitely
large to compute that.

```
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

Wrapping the recursive occurrence in [`Box<T>`](../ownership-borrowing/smart-pointers-box.md)
breaks the infinite-size problem: a `Box` is always exactly one
pointer-sized value regardless of what it points to, so `List` above has
a fixed, computable size (an enum discriminant plus the larger of its
variants, one of which is just a pointer) even though it logically
contains an unbounded chain of itself. This is the standard, idiomatic
way to write linked lists, trees, and other self-referential data
structures in Rust — the indirection through the heap is exactly what
makes the recursion possible to represent at all.

## Embedded Rust Notes

**Partial support.** `Box<T>` lives in `alloc` and needs a configured
allocator. Without one, a genuinely recursive/self-referential structure
is usually reworked around a fixed-depth array-backed arena, an index
into a `heapless::Vec` acting as a pseudo-pointer, or simply avoided in
favor of an iterative, fixed-size design — common in embedded code
where unbounded recursion/allocation is itself undesirable regardless of
whether an allocator is present.

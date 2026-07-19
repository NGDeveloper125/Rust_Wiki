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

## Basic usage example

```
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use List::{Cons, Nil};

let list = Cons(1, Box::new(Cons(2, Box::new(Nil)))); // <- Box gives each Cons a fixed, known size
```

**Restriction:** each level of nesting is a real heap allocation, and
dropping a very long chain recurses one stack frame per element — an
extremely deep list can overflow the stack on drop, which is why
production code often reworks deeply recursive structures into an
iterative form instead.

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

Matching directly on a `Box`-recursive enum reads no differently than
matching any other enum — the compiler auto-derefs through the `Box` at
each step of the recursion.

```
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use List::{Cons, Nil};

fn sum(list: &List) -> i32 {
    match list { // <- recurses through the Box at each step, one match arm per shape
        Cons(value, rest) => value + sum(rest), // <- rest: &Box<List>, matched and recursed into directly
        Nil => 0,
    }
}

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
println!("{}", sum(&list)); // 6
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) walks
through this exact `Cons`-list shape — matching straight through the
`Box` keeps the recursive walk as simple to read as any non-boxed enum,
despite the heap indirection each `Cons` involves underneath.

## Embedded Rust Notes

**Partial support.** `Box<T>` lives in `alloc` and needs a configured
allocator. Without one, a genuinely recursive/self-referential structure
is usually reworked around a fixed-depth array-backed arena, an index
into a `heapless::Vec` acting as a pseudo-pointer, or simply avoided in
favor of an iterative, fixed-size design — common in embedded code
where unbounded recursion/allocation is itself undesirable regardless of
whether an allocator is present.

---
title: "Copy vs Clone"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Move Semantics"]
related_syntax: []
see_also: ["Move semantics", "Derivable traits (Debug, Clone, PartialEq, …)"]
---

## Explanation

`Copy` and `Clone` are both about duplicating a value, but they represent
opposite philosophies about when duplication should be implicit versus
explicit.

A type that implements `Copy` is duplicated automatically, silently,
every time it would otherwise be moved — assigning it, passing it to a
function, anything. This is only allowed for types where duplication is
trivial and cheap: a bitwise copy with no heap allocation, no reference
counting, nothing that could meaningfully "go wrong" or cost more than
copying a few machine words. Simple types like integers, floats, `bool`,
`char`, and tuples/arrays/structs composed entirely of `Copy` types
qualify; anything that owns a heap allocation (`String`, `Vec<T>`, `Box<T>`)
cannot be `Copy`, because a bitwise copy of it would produce two owners of
the same heap memory — exactly what move semantics exists to prevent.

`Clone` is the explicit counterpart: calling `.clone()` produces a deep
duplicate, however expensive that is for the type in question (allocating
a whole new backing buffer for a `Vec`, incrementing a reference count for
an `Rc`). Because it's a visible method call rather than something that
happens silently on assignment, `Clone` makes potentially-costly
duplication something you can see in the code, which matters for
reasoning about performance — a `.clone()` scattered through a hot loop is
immediately visible as a candidate for a closer look, in a way an
implicit copy in a GC'd language typically isn't.

## Basic usage example

```
let a = 5;
let b = a; // <- i32 is Copy: a is silently duplicated, both remain usable
println!("{a} {b}");

let s1 = String::from("hi");
let s2 = s1.clone(); // <- String is not Copy: duplication must be explicit
println!("{s1} {s2}");
```

**Restriction:** `Copy` can only be implemented for a type if every one
of its fields is also `Copy` — any type owning a heap allocation (like
`String`) can never be `Copy`, only `Clone`.

## Embedded Rust Notes

**Full support.** `Copy` and `Clone` are both defined in `core` — no
`std` dependency. `Copy` types are especially convenient in embedded code
since they avoid any question of allocation entirely.

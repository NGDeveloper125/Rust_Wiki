---
title: "*"
kind: operator
embedded_support: full
groups: [Basics, "Ownership & Borrowing", "Memory & Unsafe / FFI"]
related_concepts: [Operator overloading, "Deref & DerefMut coercion", "Smart pointers (Box<T>)"]
related_syntax: ["*=", "&"]
see_also: ["&", "*="]
---

## Explanation

`*` has three unrelated meanings depending on position:

1. **Binary multiplication** (`a * b`) — overloadable via `std::ops::Mul`.
2. **Prefix dereference** (`*ref`) — follows a reference or smart pointer
   to the value it points to. Overloadable via `std::ops::Deref`
   (and `DerefMut` for `*ref = value`), which is exactly the mechanism
   that lets `Box<T>`, `Rc<T>`, etc. be dereferenced with plain `*`.
   Most of the time you don't need to write `*` explicitly, because
   **auto-deref** kicks in for method calls (`my_box.method()` inserts
   the derefs for you) — explicit `*` shows up mainly with operators,
   pattern matching, or assigning through a reference.
3. **Raw pointer type** (`*const T`, `*mut T`) — appears only in a type
   position, never as an expression on its own; dereferencing a raw
   pointer (`*ptr`) requires an `unsafe` block, unlike dereferencing `&T`.

Rust's grammar disambiguates these purely by position: `*` before an
expression with nothing on its left is prefix (dereference); `*` between
two expressions is binary (multiplication); `*const`/`*mut` immediately
followed by a type is the raw pointer type former.

## Embedded Rust Notes

**Full support** for all three meanings. `Mul`/`Deref` live in `core::ops`,
and raw-pointer dereference is exactly how embedded code reads/writes
memory-mapped hardware registers via `unsafe { *addr }` or, more often, a
volatile wrapper around it.

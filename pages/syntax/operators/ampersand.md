---
title: "&"
kind: operator
embedded_support: full
groups: [Basics, "Ownership & Borrowing"]
related_concepts: ["Borrowing (shared references)", "Mutable borrowing", Operator overloading]
related_syntax: ["&mut", "*", "&&", "&="]
see_also: ["*", "&mut"]
---

## Explanation

`&` has two unrelated meanings, separated by position:

1. **Prefix: borrow.** `&expr` produces a shared reference to `expr`
   without taking ownership of it; `&type` (as in `&i32`, `&'a str`) is
   the *type* of such a reference. This is the far more common use in
   everyday Rust code, and is covered in depth on the Borrowing concept
   page — the syntax angle is just: `&` creates a reference, `*`
   (see [`*`](asterisk.md)) follows one back to its target.
2. **Binary: bitwise AND.** `a & b` between two integers, overloadable via
   `std::ops::BitAnd`. Also appears in trait-bound-adjacent contexts as
   part of `&` reference types combined with lifetimes: `&'a mut T`.

`&mut expr` / `&mut Type` is the mutable-borrow counterpart, but it is
its own two-keyword combination rather than a separate single token —
see [`mut`](../keywords/mut.md).

`&&` is a distinct token (see [`&&`](ampersand-ampersand.md)), not two
`&` read together, though `&&expr` (a reference to a reference) is valid
and does get lexed as `&` `&` `expr` in that specific position — a rare
case where the lexer has to pick between the `&&`-token and two
`&`-tokens based on what follows.

## Embedded Rust Notes

**Full support** for both meanings. Borrowing is core-language and used
constantly for peripheral/driver references; `BitAnd` lives in
`core::ops` and register-mask manipulation (`status & FLAG_BIT`) is one
of the most common operations in register-level embedded code.

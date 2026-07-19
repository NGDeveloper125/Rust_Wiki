---
title: "Weak references (Weak<T>)"
area: "Ownership & Borrowing"
embedded_support: partial
groups: ["Ownership & Borrowing", "Reference Counting", "Sharing & Mutating Data Safely"]
related_syntax: []
see_also: ["Shared ownership (Rc & Arc)"]
---

## Explanation

`Weak<T>` is a non-owning companion to `Rc<T>`/`Arc<T>`: holding a `Weak`
reference to a value doesn't keep it alive and doesn't count toward its
strong-reference count, only its separate weak count. To actually use the
value, you `.upgrade()` a `Weak<T>` into an `Option<Rc<T>>` (or
`Option<Arc<T>>`) — `Some` if the value is still alive, `None` if every
strong owner has already dropped it.

This exists specifically to break reference cycles. Two `Rc`-owned
structures that hold strong references to each other (a parent pointing
at its child, and that child pointing back at its parent) would otherwise
never reach a reference count of zero — each keeps the other alive
forever, a memory leak reference counting can't detect or prevent on its
own. Making one direction of such a cycle a `Weak` reference instead (very
commonly: children hold a strong reference to a parent's data, parents
hold only a `Weak` reference back down to children) breaks the cycle
while still letting either side reach the other when needed.

## Embedded Rust Notes

**Partial support.** Same caveat as
[Shared ownership (Rc & Arc)](shared-ownership-rc-arc.md) — `Weak<T>`
lives in `alloc`, requiring a configured global allocator. Not available
at all in a bare `#![no_std]` project with no allocator.

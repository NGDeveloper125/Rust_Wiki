---
title: "Stack vs heap allocation"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Boxing", "Systems / Low-Level Programming"]
related_syntax: []
see_also: ["Smart pointers (Box<T>)", "Memory layout & repr"]
---

## Explanation

The stack is a fast, automatically-managed region of memory for values
whose size is known at compile time and whose lifetime follows the call
stack exactly — local variables in a function are pushed on entry and
popped on return, with no allocation bookkeeping required. The heap is a
separately-managed region for values whose size isn't known until
runtime, or that need to outlive the specific function call that created
them; using it means an explicit allocation (and, in Rust, an equally
explicit deallocation when the owner is dropped).

Rust puts values on the stack by default and only moves data to the heap
when you ask for it — via `Box<T>`, `Vec<T>`, `String`, `Rc`/`Arc`, or any
other type that internally allocates. This is a deliberate design choice:
stack allocation is close to free, so the language doesn't hide heap
allocations behind implicit boxing the way some higher-level languages do
for every non-primitive value — in Rust, if a type allocates, that's
usually visible either in its name (`Box`, `Vec`, `String`) or documented
behavior, not a hidden cost baked invisibly into ordinary variable use.

Knowing which one a value lives on matters for two very different
reasons: performance (stack allocation and deallocation cost is
effectively zero; heap allocation goes through an allocator and costs
real, measurable time), and what's possible at all (a
[recursive type](recursive-types-via-box.md) or a
[trait object](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
*requires* heap indirection, because their size isn't knowable at compile
time the way a stack allocation requires).

## Basic usage example

```
let a = 5;           // stack: fixed size, popped automatically at scope end
let b = Box::new(5); // <- heap: explicit allocation, freed when `b` is dropped
println!("{a} {b}");
```

## Best practices & deeper information

### Scenario: Boxing and heap allocation

A large, rarely-used buffer stored inline in an enum variant forces every
variant of that enum to reserve the biggest one's stack space — boxing
the large variant keeps the common ones cheap.

```
struct Board {
    cells: [[u8; 64]; 64], // 4 KiB — fine on its own, expensive to carry in every enum variant
}

enum GameState {
    Menu,
    Loading(u8),
    Playing(Box<Board>), // <- PREFER: heap-allocated; GameState's size is one pointer, not 4 KiB, even for `Menu`
}

let state = GameState::Playing(Box::new(Board { cells: [[0; 64]; 64] }));
```

**Why this way:** an enum's stack size is the size of its largest
variant — inlining a large `Board` directly would force `Menu` and
`Loading` to reserve the same 4 KiB on the stack even though they never
use it. Boxing the large variant is the standard fix, covered in the
[Rust Book](https://doc.rust-lang.org/book/ch15-01-box.html) as one of
`Box`'s core use cases; see
[Smart pointers (Box<T>)](smart-pointers-box.md) for the fuller
treatment.

## Embedded Rust Notes

**Full support** — and a genuinely central concern in embedded Rust.
Stack allocation requires no allocator at all and is available on every
target; many embedded projects run with **no heap configured whatsoever**,
using only stack allocation and `'static` storage, precisely to get fully
deterministic, bounded memory behavior with no possibility of allocation
failure at runtime.

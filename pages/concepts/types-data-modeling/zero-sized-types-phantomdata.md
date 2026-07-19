---
title: "Zero-sized types & PhantomData"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Designing Robust Data Models"]
related_syntax: []
see_also: ["Unit structs", "\"Make invalid states unrepresentable\""]
---

## Explanation

A zero-sized type occupies no memory at runtime at all — `size_of::<T>() == 0`
— while still existing fully as a type at compile time. Unit structs
(`struct Marker;`) and field-less enum variants are the most common
naturally-occurring examples: the compiler doesn't need to store
anything to represent a value that carries no data, since there's only
ever one possible value of that type.

`PhantomData<T>` is a special zero-sized type used to tell the compiler
"pretend this struct owns/relates to a `T`" without actually storing a
`T` anywhere in the struct — needed when a generic parameter is used only
in a way the compiler can't see directly (for example, in raw pointers
inside an `unsafe` implementation), so that lifetime checking, variance,
and drop-check analysis still treat the struct as if it genuinely
contained a `T`.

Both are examples of a broader theme: using the type system to carry
information that has real compile-time meaning but zero runtime cost —
the compiler tracks and enforces it, and none of it survives into the
compiled binary as actual bytes.

## Embedded Rust Notes

**Full support.** No allocator dependency — `PhantomData`-based typestate
is a signature embedded HAL pattern (e.g. `Pin<MODE>` encoding a GPIO
pin's configuration in its type so misusing it is a compile error, with
zero runtime representation for the state itself).

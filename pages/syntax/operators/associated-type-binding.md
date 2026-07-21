---
title: "Type<Ident = Type>"
kind: operator
embedded_support: full
groups: ["Types & Data Structures", Basics, "Traits & Polymorphism"]
related_concepts: ["Associated types", "Generics", "Trait bounds"]
related_syntax: ["<", ">", "::"]
see_also: ["<", ">"]
---

## Explanation

An associated-type binding, `Trait<AssocName = ConcreteType>`, constrains
*which* concrete type a trait's associated type resolves to at a
particular use site. It appears inside the same angle-bracket list as any
ordinary generic type arguments a trait takes, but uses `Name = Type`
syntax instead of a bare positional type — `T: Iterator<Item = u32>`
reads as "`T` must implement `Iterator`, **and** its `Item` must
specifically be `u32`," which is a stronger constraint than `T:
Iterator` alone, which leaves `Item` free to be whatever the concrete `T`
happens to produce.

The binding can appear anywhere a trait bound can: an inline bound on a
generic parameter (`fn print_all<T: Iterator<Item = String>>(items: T)`),
a `where` clause, a `dyn Trait<...>` trait-object type, or an `impl
Trait<...>` opaque type in argument or return position. For a trait with
only one associated type and no other generic parameters — `Iterator` is
the everyday example — the binding is often the *only* thing inside the
angle brackets. When a trait takes both ordinary generic type parameters
and has associated types, the positional type arguments come first and
the named bindings follow: `Trait<T, AssocName = U>`.

This syntax is not optional decoration around `dyn` trait objects: a bare
`dyn Iterator` doesn't pin down what `next()` yields, so the type isn't
concrete enough to exist as a trait object on its own — `Box<dyn
Iterator<Item = u32>>` is the form that actually compiles, because the
binding supplies the missing piece the compiler needs to know the
trait object's shape.

Deciding when an associated type is the right tool for a trait at all —
versus an ordinary generic parameter a caller would choose instead — is
covered on the
[Associated types](../../concepts/types-data-modeling/associated-types.md)
concept page; this page covers only the constraint syntax used once a
trait already has one.

## Basic usage example

```
fn print_all<T: Iterator<Item = String>>(items: T) {
    // <- `Item = String` binds T's associated type to exactly String
    for item in items {
        println!("{item}");
    }
}
```

## Best practices & deeper information

### Scenario: Writing generic code

A function that sums an iterator's values only compiles for iterators
whose `Item` is actually summable as `u32` — the associated-type binding
in the bound is what pins that down, rather than accepting any `Iterator`
and hoping the element type works out.

```
fn total_readings<I>(readings: I) -> u32
where
    I: Iterator<Item = u32>, // <- binds Item to u32; without this, Item could be anything
{
    readings.sum()
}

let total = total_readings(vec![10, 20, 30].into_iter());
```

**Why this way:** leaving off `Item = u32` and writing just `I: Iterator`
wouldn't compile inside `.sum()`, since the compiler would have no
concrete element type to sum — the
[Book's generics chapter](https://doc.rust-lang.org/book/ch10-01-syntax.html)
covers trait bounds as the mechanism for telling the compiler exactly
which operations a generic parameter supports, and an associated-type
binding is how that extends to a trait's associated types specifically.

### Scenario: Runtime polymorphism

A plugin system that hands back a boxed, dynamically-dispatched iterator
needs the binding to make `dyn Iterator` into a concrete, sized-behind-a-
pointer type in the first place.

```
fn even_numbers(limit: u32) -> Box<dyn Iterator<Item = u32>> {
    // <- `Item = u32` here is required: `dyn Iterator` alone isn't a complete type
    Box::new((0..limit).filter(|n| n % 2 == 0))
}

let evens: Vec<u32> = even_numbers(10).collect();
```

**Why this way:** returning `Box<dyn Iterator<Item = u32>>` lets the
function's actual iterator type (a `Filter<Range<u32>, _>` closure type,
here) stay hidden behind the trait object, at the cost of one allocation
and a vtable dispatch per call — the
[Rust Reference](https://doc.rust-lang.org/reference/paths.html#paths-in-expressions)
documents associated-type bindings as part of a trait object type's
grammar, not an optional add-on to it.

## Embedded Rust Notes

**Full support.** Associated-type bindings are a purely compile-time
constraint with no `std`/allocator dependency on their own; the boxed
`dyn Trait<Assoc = T>` form specifically needs `alloc` (or a `heapless`
equivalent) for the allocation `Box::new` performs, the same caveat as
any other `Box<dyn Trait>` usage.

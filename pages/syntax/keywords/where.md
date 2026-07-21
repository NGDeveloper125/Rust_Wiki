---
title: "where"
kind: keyword
embedded_support: full
groups: ["Traits & Polymorphism"]
related_concepts: ["Trait bounds", Generics]
related_syntax: [":", "+", fn, trait]
see_also: ["Trait bounds"]
---

## Explanation

`where` introduces a clause that lists trait bounds separately from a
function, struct, enum, impl, or trait's parameter list, as an
alternative to writing them inline after `:`. `fn summarize<T:
std::fmt::Debug + Clone>(items: &[T]) -> String` and `fn
summarize<T>(items: &[T]) -> String where T: std::fmt::Debug + Clone`
declare exactly the same bound on `T` — one inline, one relocated to a
`where` clause after the parameter list and before the body's opening
brace.

For simple bounds, the two forms are purely a readability choice — moving
bounds to `where` keeps the parameter list itself scannable once it
carries more than one or two constraints, or once multiple type
parameters each need their own bounds. See
[Trait bounds](../../concepts/traits-polymorphism/trait-bounds.md) for
the fuller guidance on when to prefer one form over the other.

`where` is not always optional, though. Some bounds have **no inline
position to write them in at all**, and `where` is the only legal syntax
for them:

- **Bounding an associated type of a generic parameter:**
  `where T::Item: Clone` — there is no `<T>` position to attach this
  bound to, since it constrains a type *reachable from* `T`, not `T`
  itself.
- **Bounding a concrete type, not a type parameter:** `where i32: From<T>`
  — the thing being bounded isn't a generic parameter in this function's
  own parameter list at all.
- **Bounding a type behind a lifetime in more complex forms**, such as
  combining a lifetime bound with an associated-type bound in one clause.

In every one of these, `T: Bound` syntax after `<T>` simply has nowhere
to go, because what's being constrained isn't `T` directly — `where` is
required, not stylistic, in these cases.

## Usage examples

### Relocating a bound to a `where` clause

```
fn largest<T>(items: &[T]) -> &T
where
    T: PartialOrd, // <- `where` relocates the bound out of `<T: PartialOrd>`
{
    let mut max = &items[0];
    for item in items {
        if item > max { max = item; }
    }
    max
}
```

### Writing generic code

A function generic over an iterator needs to bound the iterator's
*yielded* type, not the iterator type itself — there's no way to write
that bound inline in the `<T>` list, so `where` is required, not a style
preference.

```
fn print_all<T>(items: T)
where
    T: IntoIterator,
    T::Item: std::fmt::Display, // <- bounding an associated type: only expressible via `where`
{
    for item in items {
        println!("{item}");
    }
}

print_all(vec![1, 2, 3]);
print_all(["west", "east"]);
```

`T::Item` names a type reachable *through* `T`, not `T`
itself, so there's no `<T::Item: Display>` position to write the bound in
— the
[Rust Reference's where-clauses section](https://doc.rust-lang.org/reference/items/generics.html#where-clauses)
confirms this associated-type-bound form is only legal inside a `where`
clause.

### Designing a public API

A function with two type parameters, each carrying more than one bound,
reads far better with the bounds moved to `where` than crammed into the
angle brackets.

```
// AVOID: bounds packed into the parameter list get hard to scan
fn merge_avoid<K: std::hash::Hash + Eq + Clone, V: Clone>(
    left: &std::collections::HashMap<K, V>,
    right: &std::collections::HashMap<K, V>,
) -> std::collections::HashMap<K, V> {
    let mut merged = left.clone();
    merged.extend(right.iter().map(|(k, v)| (k.clone(), v.clone())));
    merged
}

// PREFER: `where` separates "what this does" from "what K and V must support"
fn merge<K, V>(
    left: &std::collections::HashMap<K, V>,
    right: &std::collections::HashMap<K, V>,
) -> std::collections::HashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone, // <- each parameter's bounds get their own line
    V: Clone,
{
    let mut merged = left.clone();
    merged.extend(right.iter().map(|(k, v)| (k.clone(), v.clone())));
    merged
}
```

The
[Rust Book](https://doc.rust-lang.org/book/ch10-02-traits.html#clearer-trait-bounds-with-where-clauses)
recommends `where` once a signature has more than one bounded type
parameter, precisely so the function name and parameter list stay
readable at a glance, with constraints listed separately underneath.

## Explanation (Embedded)

`where` means exactly the same thing under `#![no_std]` — a pure
compile-time clause with no runtime cost or allocator dependency. It
shows up constantly in embedded driver code specifically because
`embedded-hal` traits lean on associated types (an SPI trait's associated
`Error` type, for instance), and bounding an associated type has no
inline position to write it in — `where` is the only legal spelling here,
same as in any other Rust code with an associated-type bound.

## Usage examples (Embedded)

### Bounding a generic driver function over embedded-hal traits

```
fn init_display<SPI, CS>(spi: &mut SPI, cs: &mut CS) -> Result<(), SPI::Error>
where
    SPI: embedded_hal::spi::SpiBus, // <- `where` bounds the bus type over the SpiBus trait
    CS: embedded_hal::digital::OutputPin,
{
    cs.set_low().ok();
    spi.write(&[0xAE])?; // display-off command, as an example
    cs.set_high().ok();
    Ok(())
}
```

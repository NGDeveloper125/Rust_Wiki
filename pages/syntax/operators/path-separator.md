---
title: "::"
kind: operator
embedded_support: full
groups: ["Modules, Crates & Visibility", "Types & Data Structures"]
related_concepts: [Modules, Crates, Generics]
related_syntax: [mod, use, crate, super, "<", ">"]
see_also: ["<", generic-fn]
---

## Explanation

`::` is the path separator. It walks the module tree (see
[Modules](../../concepts/modules-crates-visibility/modules.md)) and a
crate's namespaces (see
[Crates](../../concepts/modules-crates-visibility/crates.md)) the same
way `/` walks directories: `std::collections::HashMap` reads as "the
`HashMap` item inside the `collections` module inside the `std` crate."
A path can be as long as the module nesting requires, and can begin with
a crate name, [`crate`](../keywords/crate.md), `self`, or
[`super`](../keywords/super.md) to anchor where in the tree it starts.

`::` also separates a type from an associated item reached through it —
an associated function, constant, or type — rather than a submodule:
`Vec::new()` is "the associated function `new` on the type `Vec`," and
this works identically whether the left side is a concrete type, a
generic type instantiated with a concrete argument (`Vec::<i32>::new()`),
or a fully qualified trait path (`<T as Trait>::method()`, used when more
than one in-scope trait defines a method of that name and the compiler
needs to be told which one — see
[Generics](../../concepts/types-data-modeling/generics.md) for why that
ambiguity comes up in generic code). Rust doesn't distinguish
"module-path colons" from "associated-item colons" at the token level —
only what precedes and follows a given `::` decides which role it plays.

### Turbofish: `::<...>`

Generic type or const arguments in **expression position** — calling a
generic function or method, not declaring one — must be introduced with
`::` before the angle brackets: `"42".parse::<i32>()`,
`Vec::<u8>::with_capacity(16)`. This `::<...>` form is nicknamed the
"turbofish," after the shape `::<>` resembles.

The `::` here is not optional decoration — it disambiguates the parse.
`<` is also the less-than operator (see [`<`](less-than.md)), so
`parse<i32>(x)` (no `::`) would parse as `(parse < i32) > (x)`, a chained
comparison, not a call with a generic argument. Prefixing with `::` tells
the parser unambiguously that what follows is a generic argument list.
Outside of expression position — a type annotation, a `fn` signature, an
`impl` header — there's no such ambiguity, since a bare `<...>` in those
positions can only mean generics, which is why turbofish is needed only
in expression position, never in a type.

## Basic usage example

```
use std::collections::HashMap; // <- `::` walks `std` -> `collections` -> `HashMap`

let scores: HashMap<&str, u32> = HashMap::new(); // <- `::` here reaches `HashMap`'s associated fn `new`
```

## Best practices & deeper information

### Scenario: Designing a public API

A public function parsing a numeric value out of a string has no other
type context to infer from, so the call needs turbofish to say which
type `parse` should target.

```
pub fn print_doubled(raw: &str) {
    let n = raw.parse::<i32>().unwrap();
    // <- `::<i32>`: nothing else in this function fixes `parse`'s target type
    println!("{}", n * 2);
}
```

**Why this way:** `parse`'s return type is generic over
[`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) and
isn't determined by any argument, so without either a type annotation on
`n` or turbofish on the call itself, the compiler has nothing to infer
the target type from — turbofish is the idiomatic choice here because it
keeps the type visible right at the call site instead of on a separate
binding.

### Scenario: Writing generic code

A generic function bound by a trait uses a fully qualified path to call
that trait's method specifically, even though the type also has an
inherent method of the same name.

```
trait Shape {
    fn area(&self) -> f64;
}

struct Square {
    side: f64,
}

impl Square {
    fn area(&self) -> f64 { // an inherent method, same name as the trait method
        self.side * self.side * 1.0
    }
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }
}

fn trait_area<T: Shape>(shape: &T) -> f64 {
    <T as Shape>::area(shape)
    // <- `::` here follows a qualified path, forcing `Shape`'s `area`, not any inherent method
}
```

**Why this way:** `<T as Shape>::area(shape)` is unambiguous regardless of
what inherent methods `T` might also have, which matters in generic code
where the concrete type behind `T` — and therefore its full method set —
isn't known when `trait_area` is written, only that it implements `Shape`.

## Embedded Rust Notes

**Full support.** `::` is pure compile-time grammar — path resolution,
associated-item lookup, and turbofish are all resolved before code
generation, with no `std` dependency either way. `core`/`alloc` paths use
exactly the same `::` syntax as `std` paths.

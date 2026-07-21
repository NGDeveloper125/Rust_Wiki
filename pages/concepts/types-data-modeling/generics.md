---
title: "Generics"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Writing Generic & Reusable Code", "Polymorphism", "Generic Programming"]
related_syntax: ["<", ">"]
see_also: ["Trait bounds", "Static dispatch & monomorphization", "Const generics"]
---

## Explanation

Generics let a type or function be written once and used with many
different concrete types, without duplicating the code for each one — a
function like `fn largest<T: PartialOrd>(items: &[T]) -> &T` works for any
type `T` that satisfies the bound, instead of needing a separate copy per
concrete type.

Here `T` stands for "some type, to be determined at each call site,"
constrained by a [trait bound](../traits-polymorphism/trait-bounds.md)
(`PartialOrd`, in this example) so the function body can rely on the
operations it actually needs. Generics are resolved entirely at compile
time — see
[static dispatch & monomorphization](../traits-polymorphism/static-dispatch-monomorphization.md)
— which means generic code has no inherent runtime overhead compared to
writing the same function by hand for each concrete type; the compiler
does that duplication for you, automatically, and specializes each copy
for its specific type.

This is the main way Rust achieves reusable, type-safe abstractions
without needing to fall back to dynamic typing or runtime type checks —
the compiler verifies, once, at the definition site plus every call site,
that every operation the generic code performs is valid for whatever
concrete type ends up substituted in.

## Basic usage example

```
fn largest<T: PartialOrd>(items: &[T]) -> &T {
//        ^^^^^^^^^^^^^^ T is a generic type, constrained to types that support `>`
    let mut best = &items[0];
    for item in items {
        if item > best { best = item; }
    }
    best
}

largest(&[3, 7, 2]);       // T = i32 here
largest(&[1.5, 0.2]);      // T = f64 here, same function definition
```

## Best practices & deeper information

### Scenario: Writing generic code

The trait bounds on a generic parameter should ask for exactly the
operations the function body uses — no more — so the function stays
usable with the widest reasonable range of types.

```
fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
//        ^^^^^^^^^^^^^^^^^^^^^ bounded to only what the body needs: ordering and cheap copies
    let mut best = items[0];
    for &item in &items[1..] {
        if item > best {
            best = item;
        }
    }
    best
}

let highest_temp = largest(&[21.5, 19.0, 24.3]);  // T = f64
let highest_score = largest(&[88, 95, 72]);        // T = i32, same function body
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-function-definitions)
covers bounding a type parameter to only what's used inside the body —
over-constraining with unused bounds (e.g. requiring `Clone` when
`Copy` already covers it) narrows which callers can use the function for
no benefit.

### Scenario: Working with collections

A small generic wrapper type, written once, can back many different
key/value pairings without duplicating the struct or its methods per
concrete type.

```
struct Cache<K, V> { // <- generic over both the key and value types, defined once
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> Cache<K, V> {
    fn new() -> Self {
        Cache { entries: Vec::new() }
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

let mut sensors: Cache<&str, f64> = Cache::new();
sensors.entries.push(("temp-1", 21.5));
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch10-01-syntax.html#in-struct-definitions)
covers generic struct definitions for exactly this case — `Cache<K, V>`
works for `Cache<&str, f64>`, `Cache<u32, String>`, or any other pairing
with no duplicated code, and monomorphization means each instantiation
runs exactly as fast as a hand-written version would.

## Embedded Rust Notes

**Full support.** Generics and monomorphization are purely a compile-time
mechanism — no `std` or allocator dependency, and no extra binary size
beyond what monomorphization already costs on any target.

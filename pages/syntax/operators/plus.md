---
title: "+"
kind: operator
embedded_support: full
groups: [Arithmetic, Basics, "Traits & Polymorphism"]
related_concepts: [Operator overloading]
related_syntax: ["+="]
see_also: ["+="]
---

## Explanation

`+` is arithmetic addition between two values of the same numeric type,
as in `let sum = 1 + 2;`. It's overloadable via `std::ops::Add` — any
type can define what `+` means for it, which is how `String + &str`
concatenation works (`Add` is implemented for `String`, consuming the
left operand by value).

`+` also has a completely unrelated meaning in **trait-bound position**,
where it combines multiple bounds/lifetimes rather than performing
arithmetic, as in `fn f<T: Clone + Debug>(x: T) { ... }` or
`fn g(x: &(dyn Trait + Send)) { ... }`. Here `+` reads as "and" — `T` must implement both `Clone` and `Debug`;
the trait object must implement `Trait` and be `Send`. This is pure
compile-time grammar with no `Add`-trait involvement at all; don't
confuse the two uses.

## Usage examples

### Adding two values

```
let sum = 1 + 2; // <- `+` adds two values
```

### Numeric computation

Summing a sensor's accumulated readings is ordinary arithmetic `+`, and
the point where it's worth reaching for a checked variant is when the
inputs could plausibly overflow the chosen type.

```
fn total_energy_wh(readings: &[u16]) -> Option<u32> {
    let mut total: u32 = 0;
    for &reading in readings {
        total = total.checked_add(reading as u32)?; // safer than bare `+` at scale
    }
    Some(total)
}

let today = [120u16, 340, 275];
assert_eq!(total_energy_wh(&today), Some(735));

let sum = 2 + 2; // <- `+` plain arithmetic addition, no overflow risk here
```

`checked_add` turns a potential overflow panic (debug)
or silent wraparound (release) into an explicit `Option`, which
[Clippy's `arithmetic_side_effects`](https://rust-lang.github.io/rust-clippy/master/#arithmetic_side_effects)
lint exists to flag when bare arithmetic operators are used on values
whose range isn't obviously safe.

### Writing generic code

In trait-bound position `+` combines requirements rather than adding
numbers — a generic function that needs a value it can both clone and
print declares both bounds joined by `+`.

```
use std::fmt::Debug;

fn log_and_duplicate<T: Clone + Debug>(value: T) -> (T, T) {
    // <- `+` here combines two trait bounds, unrelated to arithmetic `+`
    println!("{value:?}");
    (value.clone(), value)
}

let (a, b) = log_and_duplicate(String::from("sensor-42"));
assert_eq!(a, b);
```

Spelling out `T: Clone + Debug` at the function
signature documents exactly what the generic code needs from its caller,
which the [Book's generics chapter](https://doc.rust-lang.org/book/ch10-02-traits.html#specifying-multiple-trait-bounds-with-the--syntax)
recommends over over-constraining with a single catch-all trait or
under-constraining and hitting compile errors deep in the function body.

## Embedded Rust Notes

**Full support.** `Add` lives in `core::ops` (re-exported as `std::ops`),
so both the arithmetic and trait-bound-combinator meanings work
identically in `#![no_std]`.

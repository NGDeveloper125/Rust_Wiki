---
title: "Copy vs Clone"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Move Semantics"]
related_syntax: []
see_also: ["Move semantics", "Derivable traits (Debug, Clone, PartialEq, …)"]
---

## Explanation

`Copy` and `Clone` are both about duplicating a value, but they represent
opposite philosophies about when duplication should be implicit versus
explicit.

A type that implements `Copy` is duplicated automatically, silently,
every time it would otherwise be moved — assigning it, passing it to a
function, anything. This is only allowed for types where duplication is
trivial and cheap: a bitwise copy with no heap allocation, no reference
counting, nothing that could meaningfully "go wrong" or cost more than
copying a few machine words. Simple types like integers, floats, `bool`,
`char`, and tuples/arrays/structs composed entirely of `Copy` types
qualify; anything that owns a heap allocation (`String`, `Vec<T>`, `Box<T>`)
cannot be `Copy`, because a bitwise copy of it would produce two owners of
the same heap memory — exactly what move semantics exists to prevent.

`Clone` is the explicit counterpart: calling `.clone()` produces a deep
duplicate, however expensive that is for the type in question (allocating
a whole new backing buffer for a `Vec`, incrementing a reference count for
an `Rc`). Because it's a visible method call rather than something that
happens silently on assignment, `Clone` makes potentially-costly
duplication something you can see in the code, which matters for
reasoning about performance — a `.clone()` scattered through a hot loop is
immediately visible as a candidate for a closer look, in a way an
implicit copy in a GC'd language typically isn't.

## Basic usage example

```
let a = 5;
let b = a; // <- i32 is Copy: a is silently duplicated, both remain usable
println!("{a} {b}");

let s1 = String::from("hi");
let s2 = s1.clone(); // <- String is not Copy: duplication must be explicit
println!("{s1} {s2}");
```

**Restriction:** `Copy` can only be implemented for a type if every one
of its fields is also `Copy` — any type owning a heap allocation (like
`String`) can never be `Copy`, only `Clone`.

## Best practices & deeper information

### Scenario: Cloning and copying

Deciding whether a type should derive `Copy` comes down to whether
duplicating it is genuinely trivial — a `Point` of two `i32`s qualifies; a
`Session` holding a `String` and a `Vec` should stay `Clone`-only so
duplication remains a visible, explicit choice.

```
#[derive(Clone, Copy)] // <- safe: two i32 fields, bitwise duplication is cheap and correct
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone)] // <- deliberately NOT Copy: duplicating a session should stay a visible choice
struct Session {
    token: String,
    permissions: Vec<String>,
}

let p1 = Point { x: 1, y: 2 };
let p2 = p1; // implicit copy: both p1 and p2 remain usable
let s1 = Session { token: "abc".into(), permissions: vec!["read".into()] };
let s2 = s1.clone(); // explicit: the cost (allocating a new String/Vec) is visible at the call site
```

**Why this way:** the
[std docs for `Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)
note that implementing `Copy` is a commitment about the type, not just a
convenience — adding a heap-owning field later would be a breaking
change, so it's best reserved for types that are genuinely trivial to
duplicate, like `Point`.

### Scenario: Working with collections

Producing a list of customer names from a slice of orders means choosing
between borrowing (`Vec<&str>`, tied to the source's lifetime) and
cloning (`Vec<String>`, independent but costing an allocation per
element).

```
struct Order {
    id: u64,
    customer: String,
}

fn names_borrowed(orders: &[Order]) -> Vec<&str> {
    orders.iter().map(|o| o.customer.as_str()).collect() // <- borrows: cheap, but tied to `orders`' lifetime
}

fn names_owned(orders: &[Order]) -> Vec<String> {
    orders.iter().map(|o| o.customer.clone()).collect() // <- clones: costs an allocation, but outlives `orders`
}
```

**Why this way:** borrow when the derived collection is used before its
source goes away, and clone only when the result genuinely needs to
outlive the source or move into another owner — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/index.html)
idiom of borrowing by default and cloning only when ownership is
genuinely required applies directly to this choice.

## Embedded Rust Notes

**Full support.** `Copy` and `Clone` are both defined in `core` — no
`std` dependency. `Copy` types are especially convenient in embedded code
since they avoid any question of allocation entirely.

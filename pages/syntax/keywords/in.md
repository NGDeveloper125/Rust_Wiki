---
title: "in"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: ["The Iterator trait"]
related_syntax: [for]
see_also: [for]
---

## Explanation

`in` binds the loop variable to the iterator source in a `for` loop, as in
`for x in 0..10 { ... }`.

Outside of `for ... in ...`, `in` appears in only one other place: the
restricted-visibility syntax `pub(in crate::some::path)`. In both spots
it is part of a fixed grammar (`for PATTERN in EXPR { BLOCK }`,
`pub(in PATH)`), never a general membership-test operator the way `in`
works in Python. Testing membership in a Rust collection is a method
call instead (`collection.contains(&value)`).

## Usage examples

### Binding a loop variable to a range

```
for x in 0..10 { // <- `in` binds `x` to each value produced by `0..10`
    println!("{x}");
}
```

**Restriction:** `in` only exists as part of fixed grammar — the
`for PATTERN in EXPR { ... }` loop and `pub(in path)` restricted
visibility — it is not a standalone membership-test operator the way
`in` works in Python.

### Working with collections

`in` binds whatever pattern precedes it to each item an iterator chain
produces — including a destructured tuple, when the source yields pairs.

```
let inventory = [("widget", 12), ("gadget", 0), ("gizmo", 5)];

for (name, count) in inventory.into_iter().filter(|&(_, c)| c > 0) {
    // <- `in` binds `(name, count)` to each item the adaptor chain yields
    println!("{name}: {count} in stock");
}
```

Destructuring directly in the pattern before `in` avoids
a separate destructuring `let` inside the loop body — the
[`Iterator::filter`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter)
adaptor narrows what's seen before `in` ever binds it. `in` itself has no
meaning outside the `for` grammar and `pub(in path)` visibility — it
isn't a general membership operator — so there's nothing further to say
about it in isolation; see [`for`](for.md) for the loop it belongs to.

## Embedded Rust Notes

**Full support.** Pure grammar, part of the `for` loop form — no `std`
dependency.

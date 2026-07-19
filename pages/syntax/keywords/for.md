---
title: "for"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: ["The Iterator trait", "IntoIterator (iter/iter_mut/into_iter)"]
related_syntax: [in, while, loop]
see_also: [in]
---

## Explanation

`for` iterates over anything that implements `IntoIterator`:

```
for item in collection {
    println!("{item}");
}
```

`for item in collection` desugars to calling `.into_iter()` on
`collection` and repeatedly calling `.next()` on the result until it
yields `None`, binding each `Some(item)` to `item` in turn. This is why
three different loop variables are common in idiomatic code depending on
what's being iterated:

- `for x in collection` — consumes `collection`, yielding owned values
  (calls `IntoIterator::into_iter` on the value itself)
- `for x in &collection` — yields shared references (`&T`), since `&C`
  implements `IntoIterator` by delegating to `.iter()`
- `for x in &mut collection` — yields mutable references (`&mut T`)

`for` is not an expression — like `while`, it always evaluates to `()` and
cannot yield a value via `break`. It accepts a loop label the same way
`while`/`loop` do.

## Embedded Rust Notes

**Full support.** Iterating a fixed-size array or a `heapless::Vec` with
`for` works exactly as it does with `std` collections — the `Iterator`
machinery lives in `core`, not `std`.

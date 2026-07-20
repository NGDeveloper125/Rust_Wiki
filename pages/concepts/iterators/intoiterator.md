---
title: "IntoIterator (iter/iter_mut/into_iter)"
area: "Iterators"
embedded_support: full
groups: ["Iterators", "Iterating & Transforming Data"]
related_syntax: [for, in]
see_also: ["The Iterator trait", "Custom iterators", "Move semantics", "Borrowing (shared references)", "Mutable borrowing"]
---

## Explanation

`IntoIterator` is the trait that turns something into an
[iterator](the-iterator-trait.md) in the first place. Its one method,
`into_iter(self) -> Self::IntoIter`, consumes the value it's called on
and returns a type implementing `Iterator`. Collections don't implement
`Iterator` directly — a `Vec` has no `next()` method of its own — they
implement `IntoIterator`, which is the entry point that hands back
something that does.

Three forms exist because there are three different ways to access data:
`iter()` borrows the source and yields shared references (`&T`),
`iter_mut()` mutably borrows it and yields exclusive references (`&mut
T`), and `into_iter()` takes ownership and yields owned values (`T`).
This mirrors exactly the choice between
[borrowing](borrowing-shared-references.md),
[mutable borrowing](mutable-borrowing.md), and
[moving](move-semantics.md) that comes up everywhere else in Rust —
`iter`/`iter_mut` are conventionally inherent methods a collection
provides, while `into_iter()` comes from implementing `IntoIterator` for
the collection itself, and separately for `&Collection` and `&mut
Collection`, which is what makes `iter()`/`iter_mut()` reachable through
`IntoIterator` too.

This trait is also what a `for` loop is built on: `for x in collection`
desugars to calling `IntoIterator::into_iter` on `collection` and
repeatedly calling `next()` on the result until it yields `None`. Writing
`for x in &collection` versus `for x in collection` versus `for x in
&mut collection` is choosing which of the three `IntoIterator`
implementations gets used, which is why the same loop syntax can yield
`&T`, `T`, or `&mut T` depending on exactly what follows `in`.

Implementing `IntoIterator` for a type you own — typically by handing out
a small [custom iterator](custom-iterators.md) struct — is what makes
`for item in my_collection` and `for item in &my_collection` work for
your own types, the same way it works for `Vec` and `HashMap`. It's worth
keeping `IntoIterator` and `Iterator` mentally separate: `IntoIterator` is
about *getting* an iterator from something, while `Iterator` is about
*being* one once you have it.

## Basic usage example

```
let names = vec!["ada".to_string(), "grace".to_string()];

for name in &names { // <- `&names` invokes IntoIterator for &Vec<T>, yielding &String
    println!("{name}");
}
```

## Best practices & deeper information

### Scenario: Sharing data with multiple references

Restocking and then shipping a warehouse inventory needs all three
`IntoIterator` forms in sequence: read-only inspection, in-place
mutation, and finally a consuming pass that hands each item off.

```
struct Item { name: String, qty: u32 }

let mut inventory = vec![
    Item { name: "bolt".into(), qty: 120 },
    Item { name: "washer".into(), qty: 0 },
];

for item in inventory.iter() { // <- borrows: caller keeps `inventory` afterward
    println!("{}: {}", item.name, item.qty);
}

for item in inventory.iter_mut() { // <- mutably borrows: can rewrite in place
    if item.qty == 0 {
        item.qty = 10; // restock
    }
}

for item in inventory.into_iter() { // <- consumes: `inventory` is gone after this loop
    println!("shipped {}", item.name);
}
```

**Why this way:** the `iter`/`iter_mut`/`into_iter` triple lets each pass
ask for exactly the access it needs — read, mutate, or take — instead of
one form doing double duty, the same discipline the
[Rust Book](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
applies to borrowing in general.

### Scenario: Working with collections

Iterating a `HashMap` by reference hands back key-value pairs without
taking ownership of the map, letting the same map be used again
afterward.

```
use std::collections::HashMap;

let mut scores: HashMap<String, u32> = HashMap::new();
scores.insert("alice".into(), 42);
scores.insert("bob".into(), 17);

for (name, score) in &scores { // <- &HashMap implements IntoIterator, yielding (&String, &u32)
    println!("{name}: {score}");
}
```

**Why this way:** `&HashMap<K, V>` implements `IntoIterator<Item = (&K,
&V)>`, so borrowing the map in the loop header is enough to read every
entry — no `.iter()` call is even required, per the
[`HashMap` docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html#impl-IntoIterator-for-%26HashMap%3CK,+V,+S%3E).

### Scenario: Writing generic code

A function that totals up quantities shouldn't force callers to build a
`Vec` first — accepting anything convertible into an iterator covers
arrays, `Vec`s, and ranges alike.

```
fn total_quantity(items: impl IntoIterator<Item = u32>) -> u32 {
    // <- accepts anything convertible into an iterator of u32
    items.into_iter().sum()
}

let from_vec = total_quantity(vec![10, 20, 30]);
let from_array = total_quantity([5, 5]);
assert_eq!(from_vec, 60);
assert_eq!(from_array, 10);
```

**Why this way:** accepting `impl IntoIterator<Item = T>` instead of
`Vec<T>` or `&[T]` gives callers the most freedom in what they pass, per
the
[API Guidelines' flexibility checklist](https://rust-lang.github.io/api-guidelines/flexibility.html).

## Embedded Rust Notes

**Full support.** `IntoIterator` lives in `core::iter` and requires no
allocator — iterating a `heapless::Vec` or a fixed-size array by
reference, mutable reference, or value works exactly as it does with
`std` collections.

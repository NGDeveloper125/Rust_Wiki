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
[borrowing](../ownership-borrowing/borrowing-shared-references.md),
[mutable borrowing](../ownership-borrowing/mutable-borrowing.md), and
[moving](../ownership-borrowing/move-semantics.md) that comes up everywhere else in Rust —
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

## Explanation (Embedded)

`IntoIterator` lives in `core::iter`, so it needs no allocator: the same
`into_iter(self) -> Self::IntoIter` contract that turns a `Vec` into
something iterable on a hosted target turns a fixed-size array or a
`heapless::Vec` into something iterable on a `#![no_std]` target. The
same three forms carry over unchanged — `iter()` borrows and yields `&T`,
`iter_mut()` mutably borrows and yields `&mut T`, `into_iter()` takes
ownership and yields `T` — and a `for` loop over a fixed array or a
`heapless` collection desugars to `IntoIterator::into_iter` exactly the
way it does over a `Vec`. This is what makes `for reading in
&sensor_buffer` or `for reading in &mut register_bank` idiomatic on
embedded targets: the loop syntax and the borrowing choice it expresses
are identical to hosted Rust, only the underlying container is a
fixed-capacity one instead of a heap-growable one.

## Basic usage example (Embedded)

```
let readings: [u16; 3] = [512, 498, 610];

for reading in &readings { // <- `&readings` invokes IntoIterator for &[u16; 3], yielding &u16
    let _ = reading; // process each ADC reading in place, no allocation
}
```

## Best practices & deeper information (Embedded)

### Scenario: Sharing data with multiple references

Reading a bank of GPIO pin states, then updating a "stale" flag in place,
then finally consuming the buffer to hand each state off to a logging
routine needs all three `IntoIterator` forms — none of which require a
heap.

```
struct PinState { pin: u8, high: bool, stale: bool }

let mut pins = [
    PinState { pin: 0, high: true, stale: false },
    PinState { pin: 1, high: false, stale: true },
];

for p in pins.iter() { // <- borrows: caller still owns `pins` afterward
    let _ = (p.pin, p.high);
}

for p in pins.iter_mut() { // <- mutably borrows: can clear or update in place
    if p.stale {
        p.stale = false; // refreshed
    }
}

for p in pins.into_iter() { // <- consumes: `pins` is gone after this loop
    let _ = p.pin; // hand each reading off, e.g. to a log buffer
}
```

**Why this way:** the same `iter`/`iter_mut`/`into_iter` triple that
disciplines borrowing on a hosted `Vec` disciplines it identically on a
plain `[PinState; 2]` array, with no allocation involved at any of the
three passes.

### Scenario: Working with collections

Iterating a `heapless::Vec` of recent readings by reference hands back
each value without taking ownership of the buffer, exactly like
iterating a `Vec` by reference would on a hosted target.

```
// [dependencies] heapless = "0.8"
use heapless::Vec;

let mut recent: Vec<u16, 8> = Vec::new();
recent.push(512).unwrap();
recent.push(498).unwrap();

for reading in &recent { // <- &heapless::Vec implements IntoIterator, yielding &u16
    let _ = reading;
}
```

**Why this way:** `heapless::Vec<T, N>` implements `IntoIterator` the
same way `std::vec::Vec<T>` does, so code written against `for x in
&collection` ports to a fixed-capacity, no-heap buffer with no change to
the loop itself.

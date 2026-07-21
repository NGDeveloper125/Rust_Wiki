---
title: "for"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics]
related_concepts: ["The Iterator trait", "IntoIterator (iter/iter_mut/into_iter)"]
related_syntax: [in, while, loop]
see_also: [in]
---

## Explanation

`for` iterates over anything that implements `IntoIterator`, repeatedly
binding the loop variable to each item the iterator produces.

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

`for` is an expression, but — like `while` — it always evaluates to `()`
and cannot yield a value via `break value;`. It accepts a loop label the
same way `while`/`loop` do.

## Basic usage example

```
for item in [1, 2, 3] { // <- `for` iterates over anything implementing `IntoIterator`
    println!("{item}");
}
```

## Best practices & deeper information

### Scenario: Working with collections

Printing only the large orders from a list reads best as `for` driving an
adaptor chain, rather than a `for` loop whose body starts with an `if`.

```
let orders = vec![42.50, 19.99, 7.25, 103.00];

for total in orders.iter().filter(|&&t| t > 20.0) {
    // <- `for` drives the adaptor chain; `filter` has already narrowed what it sees
    println!("large order: {total}");
}
```

**Why this way:** letting `filter` express the condition keeps the loop
body focused on what to *do* with each item rather than whether to do it
— the
[`Iterator` trait docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
are themselves written around composing adaptors this way.

### Scenario: Message passing between threads

A worker thread sends results over a channel and then finishes; the
receiving side drains everything sent with a plain `for` loop, which ends
automatically once every sender has been dropped.

```
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    for id in [1, 2, 3] {
        tx.send(format!("order {id} processed")).unwrap();
    }
}); // `tx` drops here when the closure returns, closing the channel

for message in rx { // <- `for` drains the receiver until the channel closes
    println!("{message}");
}
```

**Why this way:** `Receiver` implements `IntoIterator` (for both
`Receiver` and `&Receiver`), so `for message in rx` blocks for the next
message and stops cleanly once all senders have dropped — the
[Book's message-passing chapter](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
uses exactly this shape instead of a manual `loop` calling `.recv()`.

## Embedded Rust Notes

**Full support.** Iterating a fixed-size array or a `heapless::Vec` with
`for` works exactly as it does with `std` collections — the `Iterator`
machinery lives in `core`, not `std`.

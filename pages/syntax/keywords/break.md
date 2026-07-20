---
title: "break"
kind: keyword
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: [loop, while, for, continue]
see_also: [continue, loop]
---

## Explanation

`break` exits the nearest enclosing loop immediately:

```
let done = true;
loop {
    if done {
        break;
    }
}
```

Inside a `loop` (but not `while`/`for`), `break` can carry a value —
`break value;` — which becomes the result of the whole `loop` expression.
This is the *only* loop form where that's legal, since `while`/`for` may
execute zero iterations and therefore can't guarantee a value exists to
break with.

To exit an outer loop from inside a nested one, label the outer loop and
target it explicitly: `break 'outer;` (see loop labels under `loop`,
`while`, `for`). `break` can also be used inside a labeled non-loop block
(`'a: { ... break 'a value; }`) to exit early with a value, a lesser-known
form useful for structuring multi-step logic without a `loop` at all.

## Basic usage example

```
let result = loop {
    break 5; // <- `break` exits the loop immediately, with `5` as its value
};
```

**Restriction:** `break value;` is legal inside `loop` and inside a
labeled block (`'a: { ... break 'a value; }`), but not in `while`/`for` —
those loops can terminate without executing any `break` (the condition
turns false or the iterator runs out), so a loop value would have no
defined result.

## Best practices & deeper information

### Scenario: Working with collections

Searching a slice of orders for one matching an id can be written as a
manual loop over an iterator so that a match arm can `break` with the
found value once it's located.

```
struct Order { id: u32, total: f64 }

let orders = [
    Order { id: 101, total: 42.50 },
    Order { id: 102, total: 19.99 },
    Order { id: 103, total: 7.25 },
];

let mut iter = orders.iter();
let found = loop {
    match iter.next() {
        Some(order) if order.id == 102 => break Some(order), // <- `break` exits `loop`, carrying the found order out as its value
        Some(_) => continue,
        None => break None,
    }
};
```

**Why this way:** for a plain predicate search,
[`Iterator::find`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.find)
is more idiomatic than a hand-rolled loop; `break value` earns its keep
once the search loop needs to do per-item work an adaptor chain can't
express as cleanly.

### Scenario: Branching on data (pattern matching)

A loop draining a queue of commands can use `break` inside one arm of a
`match` to stop only on a specific variant, while other variants keep the
loop running.

```
enum Command { Process(u32), Retry, Shutdown }

let mut queue = vec![Command::Process(7), Command::Retry, Command::Shutdown];
let mut processed = 0;

while let Some(cmd) = queue.pop() {
    match cmd {
        Command::Process(id) => processed += id,
        Command::Retry => continue,
        Command::Shutdown => break, // <- exits the loop only when this arm matches; other arms keep looping
    }
}
```

**Why this way:** scoping the exit condition to one `match` arm reads more
directly than a boolean flag checked after the match, and the
[Reference's loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html)
confirm `break` is legal from any position textually inside the loop body,
including nested inside a `match` — with two exceptions: it cannot cross
a closure or `async` block boundary to exit an outer loop, and an
unlabeled `break` always targets the nearest enclosing loop.

## Embedded Rust Notes

**Full support.** No `std` dependency; works identically in `#![no_std]`.

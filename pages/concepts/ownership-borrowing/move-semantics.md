---
title: "Move semantics"
area: "Ownership & Borrowing"
embedded_support: full
groups: ["Ownership & Borrowing", "Move Semantics", "Unique to Rust", "Coming from C / C++"]
related_syntax: [move, "="]
see_also: ["Ownership", "Copy vs Clone"]
---

## Explanation

Assigning a value to a new variable, passing it to a function, or
returning it from one transfers ownership rather than copying it by
default — this is a **move**. After a move, the original binding is no
longer valid; the compiler tracks this and rejects any later use of it as
a compile error, not a runtime bug.

```
let a = String::from("hi");
let b = a;      // ownership moves from a to b
// using `a` here is a compile error: value moved
```

This is a deliberate departure from two more familiar defaults: it's not
implicit reference/pointer semantics (as in Python, Java, JS, where
assignment shares the same object and mutation is visible through every
reference to it), and it's not implicit copying (as in C++, where
`Foo b = a;` invokes a copy constructor by default, unless you explicitly
write `std::move(a)`). Rust flips the C++ default: moving is the norm,
and copying only happens when a type explicitly opts in via `Copy` (see
[Copy vs Clone](copy-vs-clone.md)) or you call `.clone()` yourself.

The benefit is that "who owns this, and is it still valid here" is always
statically knowable and enforced by the compiler — there's no way to
accidentally hold onto and use a value that's already been logically
handed off elsewhere, a whole category of bug (use-after-move,
double-free) that move semantics eliminates by construction rather than
by convention or discipline.

## Basic usage example

```
fn consume(s: String) {
    println!("{s}");
} // s is dropped here, at the end of consume's scope

let a = String::from("hi");
consume(a); // <- ownership of `a` moves into the function call
// println!("{a}"); // would fail to compile: `a` was moved
```

## Best practices & deeper information

### Scenario: Transferring ownership

A function that consumes a `String` to build a `Report` should take it by
value, not by reference — the report becomes the text's new owner, and
the signature says so.

```
struct Report {
    body: String,
}

fn finalize(body: String) -> Report { // <- takes ownership: `body` is moved in, not borrowed
    Report { body }
}

let draft = String::from("quarterly summary");
let report = finalize(draft); // <- `draft` moves here
// println!("{draft}"); // would fail to compile: draft was moved into finalize
println!("{}", report.body);
```

**Why this way:** taking `String` by value instead of `&str` is the right
signature when the callee needs to keep the data — moving avoids an
unnecessary clone, and the compiler statically stops the caller from
reusing a value it no longer owns, which the
[Rust Book](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
highlights as move semantics' main benefit over languages with implicit
sharing.

### Scenario: Multi-threading

Spawning a worker thread that might outlive the function that started it
means the thread needs to own its input data for its whole lifetime, not
just borrow it.

```
use std::thread;

let batch = vec![1, 2, 3, 4, 5];

let handle = thread::spawn(move || { // <- `move` forces `batch` to move into the closure, not borrow it
    let sum: i32 = batch.iter().sum();
    println!("batch sum: {sum}");
});

// batch is no longer usable here: ownership moved into the spawned thread
handle.join().unwrap();
```

**Why this way:** `thread::spawn` can't prove the spawned thread finishes
before the caller's local variables go out of scope, so it requires
`'static` data — moving ownership into the closure with `move` is how a
value that isn't already `'static` becomes safe to hand to an
independently-running thread; the
[Rust Book](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
covers this as the standard way to share owned data with a spawned
thread.

## Embedded Rust Notes

**Full support.** Move semantics are core-language and allocator-free —
identical behavior in `#![no_std]`.

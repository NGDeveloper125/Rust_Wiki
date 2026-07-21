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
a trivial, bitwise copy: no heap allocation, no reference counting,
nothing that could meaningfully "go wrong". (Note that `Copy` does not
imply *small* — a `[u8; 1_000_000]` is `Copy`, and copying it is a real
megabyte-sized memcpy, so large arrays are the exception to "cheap".)
Simple types like integers, floats, `bool`,
`char`, and tuples/arrays/structs composed entirely of `Copy` types
qualify; anything that owns a heap allocation (`String`, `Vec<T>`, `Box<T>`)
cannot be `Copy`, because a bitwise copy of it would produce two owners of
the same heap memory — exactly what move semantics exists to prevent.

`Clone` is the explicit counterpart: calling `.clone()` produces whatever
duplication the type defines — a deep copy for `Vec` or `String`
(allocating a whole new backing buffer), but a cheap reference-count
increment for `Rc`/`Arc`. Because it's a visible method call rather than something that
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

## Explanation (Embedded)

`Copy` and `Clone` mean exactly the same thing under `#![no_std]` — both
traits live in `core`, so nothing about the mechanism itself changes; a
`Clone` impl for something like a `heapless::Vec` still copies real bytes
around, it just never touches an allocator to do it. What genuinely
differs is the cost asymmetry: on a device with a few kilobytes of RAM and
no allocator, a `.clone()` on anything beyond a handful of bytes is a
real, visible cost in stack space and cycles, not the "cheap enough not
to think about" operation it usually is on a desktop with megabytes of
cache. This tilts the usual `Copy`/`Clone` choice further toward `Copy`
for embedded's characteristic types — register-value wrappers, small
sensor readings, timestamps — since a handful of bytes is exactly what
`Copy`'s free bitwise duplication is for, and it removes the temptation to
reach for `.clone()` reflexively later on something bigger. See
[Anti-pattern: cloning to satisfy the borrow
checker](../design-patterns-idioms/anti-pattern-clone-to-satisfy-borrow-checker.md)
for that reflex's specific failure mode — its embedded stakes are higher
still, since a clone added just to silence the borrow checker can
duplicate a `heapless`-style fixed-capacity buffer that the surrounding
code budgeted RAM for exactly once.

## Basic usage example (Embedded)

```
#[derive(Clone, Copy)]
struct AdcSample { channel: u8, value: u16 } // <- 3 bytes: bitwise copy is free, Copy is the right call

let a = AdcSample { channel: 0, value: 512 };
let b = a; // <- silently duplicated, both usable — no cost beyond the 3 bytes already on the stack
println!("{} {}", a.value, b.value);
```

## Best practices & deeper information (Embedded)

### Scenario: Cloning and copying

A small register-value sample type derives `Copy`; a log built on a
`heapless` buffer stays `Clone`-only, because duplicating it copies a real,
visible chunk of the device's RAM budget.

```
#[derive(Clone, Copy)] // <- 3 bytes total: bitwise duplication is free, no reason to make it explicit
struct AdcSample { channel: u8, value: u16 }

#[derive(Clone)] // <- deliberately NOT Copy: duplicating this copies the whole fixed-capacity buffer
struct SensorLog {
    samples: heapless::Vec<AdcSample, 32>, // <- up to 32 * 3 bytes moved on every .clone()
}

let sample = AdcSample { channel: 0, value: 512 };
let sample2 = sample; // <- implicit copy: 3 bytes, effectively free
let log = SensorLog { samples: heapless::Vec::new() };
let log_snapshot = log.clone(); // <- explicit: visibly copies up to 96 bytes, worth a second look in a hot path
```

**Why this way:** on a device with only a few KB of RAM total, an implicit
`Copy` staying cheap (a handful of bytes) versus an explicit `.clone()`
that can move nearly a hundred bytes is exactly the distinction `Clone`'s
visibility is meant to surface — the
[std docs for `Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html)
frame deriving it as a commitment that's only appropriate for genuinely
trivial types, which keeps every silent duplication in the program free
while anything `heapless`-backed stays visibly costed at the call site.

### Scenario: Working with collections

Reading the most recent value out of a fixed-capacity log collapses the
usual borrow-vs-clone tradeoff into a near-free decision once the element
type is small and `Copy`.

```
fn latest_borrowed(log: &heapless::Vec<AdcSample, 32>) -> Option<&AdcSample> {
    log.last() // <- borrows: zero-cost, tied to `log`'s lifetime
}

fn latest_owned(log: &heapless::Vec<AdcSample, 32>) -> Option<AdcSample> {
    log.last().copied() // <- AdcSample is Copy: ".copied()" is a 3-byte copy, not a heap-owning clone
}
```

**Why this way:** because `AdcSample` is `Copy`, pulling one out of the
log with `.copied()` costs exactly 3 bytes and never risks an allocation —
the same borrow-vs-own choice the classic page makes between
`Vec<&str>`/`Vec<String>` collapses to a near-free decision once the
element type is small and `Copy`, which is exactly why embedded code
favors small `Copy` types for anything read out of a collection often.

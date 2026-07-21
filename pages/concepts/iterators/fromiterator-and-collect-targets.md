---
title: "FromIterator & collect targets"
area: "Iterators"
embedded_support: partial
groups: ["Iterators", "Functional Programming", "Iterating & Transforming Data"]
related_syntax: []
see_also: ["The Iterator trait", "IntoIterator (iter/iter_mut/into_iter)", "Iterator consumers", "Generics"]
---

## Explanation

`FromIterator` is the trait a type implements to say "I know how to be
built from an iterator." Its one method, `from_iter`, takes anything
implementing [`Iterator`](the-iterator-trait.md) and produces a value of
the implementing type. `Iterator::collect` is really just a thin,
convenient wrapper around this: calling `.collect::<B>()` calls
`B::from_iter(self)` — all the actual "how do I build a `B` out of these
items" logic lives in `B`'s `FromIterator` implementation, not in
`collect` itself.

The question "how does `.collect()` know what to build?" is answered by
ordinary type inference: the compiler looks at how the result is used —
a `let` binding's type annotation, a function's declared return type, or
an explicit turbofish like `.collect::<Vec<_>>()` — and picks whichever
`FromIterator` implementation matches. Many standard types implement it:
`Vec<T>`, `String` (from `char`s or `&str`s), `HashMap`/`BTreeMap` (from
`(K, V)` tuples), `HashSet`/`BTreeSet`, and — easy to overlook but
genuinely useful — `Result<Vec<T>, E>` and `Option<Vec<T>>`, which let
`collect` short-circuit and stop at the first `Err` or `None` in the
source iterator instead of requiring a manual loop with an early return.

`FromIterator` is effectively the return leg of the round trip
[`IntoIterator`](intoiterator.md) starts: `IntoIterator` turns a
collection into an iterator, and `FromIterator` turns an iterator back
into a collection (possibly a different one than it started as — this is
exactly how `.chars().collect::<Vec<char>>()` turns a `&str` into a
`Vec<char>`). Implementing `FromIterator` for one of your own types is
what makes it a valid `.collect()` target the same way the standard
types are.

`collect` is technically one of the [consumers](iterator-consumers.md) —
it drives iteration to completion just like `sum` or `count` — but it
earns its own page because *what* gets built is a much bigger design
space than a single accumulated value. Many `FromIterator`
implementations (`Vec`'s in particular) use the iterator's `size_hint` to
pre-allocate the right capacity up front, so `collect` is rarely just a
naive `push`-in-a-loop under the hood.

## Basic usage example

```
let words = ["ready", "set", "go"];

let sentence: String = words.iter().copied().collect(); // <- collect() calls String::from_iter under the hood
assert_eq!(sentence, "readysetgo");
```

## Best practices & deeper information

### Scenario: Working with collections

Pairing up usernames with scores and collecting straight into a
`HashMap` skips a manual loop of `insert` calls entirely.

```
use std::collections::HashMap;

let usernames = ["alice", "bob", "carol"];
let scores = [42, 17, 88];

let leaderboard: HashMap<&str, i32> = usernames
    .into_iter()
    .zip(scores)
    .collect(); // <- collect() infers HashMap::from_iter from the binding's type

assert_eq!(leaderboard["bob"], 17);
```

**Why this way:** `HashMap`'s `FromIterator` implementation accepts any
iterator of `(K, V)` tuples, which is exactly what `zip` produces, so
`collect` builds the whole map in one expression — the
[`HashMap` docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html#impl-FromIterator%3C(K,+V)%3E-for-HashMap%3CK,+V,+RandomState%3E)
document this impl directly.

### Scenario: Working with text

Filtering the digits out of a hint string and collecting them straight
into a `String` needs no intermediate `Vec<char>` at all.

```
let raw_password_hint = "p4ssw0rd123";

let digits_only: String = raw_password_hint
    .chars()
    .filter(|c| c.is_ascii_digit())
    .collect(); // <- collect() infers String::from_iter here because the binding is typed String

assert_eq!(digits_only, "40123");
```

**Why this way:** `String` implements `FromIterator<char>`, so
filter-then-collect is the idiomatic way to build a new `String` from a
subset of another's characters, rather than a manual loop pushing into a
`String::new()`, per the
[API Guidelines' conversion conventions](https://rust-lang.github.io/api-guidelines/interoperability.html).

### Scenario: Handling and propagating errors

Parsing a batch of configured port numbers should fail on the first bad
one rather than silently dropping it — collecting into a `Result<Vec<_>,
_>` does exactly that.

```
fn parse_port(raw: &str) -> Result<u16, std::num::ParseIntError> {
    raw.trim().parse()
}

let raw_ports = ["8080", "3000", "9090"];

let ports: Result<Vec<u16>, _> = raw_ports
    .iter()
    .copied()
    .map(parse_port)
    .collect(); // <- collect() targets Result<Vec<u16>, _>, short-circuiting on the first Err

assert_eq!(ports, Ok(vec![8080, 3000, 9090]));
```

**Why this way:** `Result<V, E>` implements `FromIterator<Result<T, E>>`
for any `V: FromIterator<T>`, so collecting a sequence of `Result`s stops
at the first `Err` and returns it, instead of needing a manual loop with
an early `return` — the
[`std::result` module docs](https://doc.rust-lang.org/std/result/#collecting-into-result)
describe this pattern directly.

## Embedded Rust Notes

**Partial support.** The `FromIterator` trait itself is defined in
`core`, but the concrete collect targets shown here need more: `Vec` and
`String` require `alloc`, and `HashMap` additionally requires `std` (for
its default hasher). On a `#![no_std]` target without `alloc`, collect
into a fixed-capacity `heapless::Vec` instead, or fold the iterator
manually into a stack-allocated buffer.

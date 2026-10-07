---
title: "either"
version: "1.19.0"
publisher: "bluss, Josh Stone (cuviper), Jack Wrenn (jswrenn)"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-07"
summary: "One enum — `Left` or `Right` — that lets a function return either of two unrelated types. It forwards `Iterator`, `Read`, `Write`, `Future` and `Deref` to whichever side it holds, so the branch disappears from the call site."
domain: "Collections & data structures"
categories: ["data-structures", "no-std"]
repository: "https://github.com/rayon-rs/either"
docs: "https://docs.rs/either/1/"
---

## Overview

`either` is one enum and a pile of methods on it:

```
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```

That is the whole data model. What makes it worth a dependency is the problem it
solves, which Rust's type system creates and does not otherwise give you a way
out of: **two branches of an `if` must have the same type.** When they produce
different concrete types, and `Box<dyn Trait>` is more than you want to pay,
`Either` is the answer.

```
use either::Either;

// -> impl Iterator would not compile: the two arms are different types.
fn numbers(reversed: bool) -> impl Iterator<Item = u32> {
    if reversed {
        Either::Left((1..=3).rev())
    } else {
        Either::Right(1..=3)
    }
}

assert_eq!(numbers(true).collect::<Vec<_>>(), vec![3, 2, 1]);
assert_eq!(numbers(false).collect::<Vec<_>>(), vec![1, 2, 3]);
```

The reason that works is the part people miss on first reading. `Either` is not
just a container you unwrap later — **it implements the traits both sides
implement, and forwards to whichever one it holds.** `Iterator`, `DoubleEnded`
and `ExactSizeIterator`, `Future`, `Read`, `Write`, `BufRead`, `Seek`, `Deref`,
`AsRef`, `Error`, `fmt::Write`. So `Either<Rev<A>, A>` is itself an iterator, and
the branch stops being visible to the caller.

**It is not a `Result`, and it deliberately does not pretend to be.** There is
no success side and no failure side, no `?`, no `Try`. `Left` and `Right` carry
no meaning beyond "the first one" and "the second one" — which is exactly what
you want for a type that models a fork in the code rather than an error. (A
`From<Result<R, E>>` conversion does exist, mapping `Err` to `Left` and `Ok` to
`Right`; it is a convenience, not a claim that the two types are the same idea.)

It is a sensible dependency to take. One crate, no required dependencies, `std`
behind a default feature so `no_std` works by turning it off, MSRV 1.63, and it
is maintained alongside `rayon` by people who have kept it stable for a decade —
the 1.x line has not broken. `serde` support is an optional feature.

**When not to reach for it:** three or more branches, where nesting
`Either<A, Either<B, C>>` gets unreadable and `Box<dyn Trait>` is clearer; and
cases where the two sides genuinely share a trait you control, where your own
enum with named variants documents the intent better than `Left` and `Right`
ever will.

## When to use it

### Use case: Returning one of two iterator pipelines

The headline case, and the one that brings most projects to the crate.

```
use either::Either;

#[derive(Clone, Copy)]
enum Filter {
    All,
    EvensOnly,
}

fn apply(values: Vec<u32>, filter: Filter) -> impl Iterator<Item = u32> {
    match filter {
        // Two different adaptor types, one return type.
        Filter::All => Either::Left(values.into_iter()),
        Filter::EvensOnly => Either::Right(values.into_iter().filter(|n| n % 2 == 0)),
    }
}

let all: Vec<_> = apply(vec![1, 2, 3, 4], Filter::All).collect();
let evens: Vec<_> = apply(vec![1, 2, 3, 4], Filter::EvensOnly).collect();

assert_eq!(all, vec![1, 2, 3, 4]);
assert_eq!(evens, vec![2, 4]);
```

**Why it fits:** `Filter` on an iterator has a type that names its closure, so
the two arms can never unify and `impl Iterator` rejects them. The alternatives
are `Box<dyn Iterator>`, which allocates and makes every `next` a virtual call,
or collecting both arms into a `Vec`, which allocates and gives up laziness.
`Either` costs neither — it is a plain enum, and the branch is one discriminant
check per `next`.

### Use case: Reading from a file or from standard input

The same shape, applied to I/O instead of iteration.

```
use either::Either;
use std::io::{BufRead, BufReader, Read, Write};

fn open(path: Option<&str>) -> std::io::Result<impl Read> {
    Ok(match path {
        Some(p) => Either::Left(std::fs::File::open(p)?),
        None => Either::Right(std::io::stdin()),
    })
}

fn main() -> std::io::Result<()> {
    let path = std::env::temp_dir().join("either-example.txt");
    std::fs::File::create(&path)?.write_all(b"from a file\n")?;

    // Either<File, Stdin> implements Read, so the caller never branches.
    let source = open(path.to_str())?;
    let mut first = String::new();
    BufReader::new(source).read_line(&mut first)?;

    assert_eq!(first.trim(), "from a file");
    Ok(())
}
```

**Why it fits:** `File` and `Stdin` share no inheritance and no sealed trait you
can name, so without `Either` this becomes `Box<dyn Read>` — or worse, the
branch leaks into every caller. `Read` is forwarded, so `BufReader`,
`read_to_string` and the rest work unchanged. The same trick covers a
`Write` that is a file or `stdout`, and a `Future` that is one of two async
paths.

### Use case: Splitting one pass into two collections

`Either` has a `FromIterator` impl for tuples, which turns a single `map` into a
partition that can change the type of each side.

```
use either::{Either, Left, Right};

let raw = ["12", "x", "7", "", "40"];

// One pass; each item decides which collection it lands in.
let (numbers, rejects): (Vec<u32>, Vec<&str>) = raw
    .into_iter()
    .map(|s| match s.parse::<u32>() {
        Ok(n) => Left(n),
        Err(_) => Right(s),
    })
    .collect();

assert_eq!(numbers, vec![12, 7, 40]);
assert_eq!(rejects, vec!["x", ""]);
```

**Why it fits:** `Iterator::partition` requires both halves to hold the same
type, so it cannot separate parsed numbers from the strings that failed.
`itertools`' `partition_map` does this too, and if you already depend on
[`itertools`](itertools.md) there is no reason to add `either` for it alone —
but `itertools` depends on `either` to express the result, so the type you get
back is this one either way.

## API map

`Either` has a large surface, but it is systematic: nearly every method comes as
a `_left` and a `_right` pair, and the names mirror `Option` and `Result`. The
entries below cover the ones that earn their place; once the pattern is clear the
rest read themselves.

### Collapsing the branch

#### `either`

Applies one of two closures and returns a single value.

```
use either::{Either, Left, Right};

let value: Either<u32, String> = Left(7);

// Both closures must return the same type.
let shown = value.either(|n| n.to_string(), |s| s);
assert_eq!(shown, "7");

let other: Either<u32, String> = Right("hello".to_string());
assert_eq!(other.either(|n| n.to_string(), |s| s), "hello");
```

**When to use it:** whenever you are done branching and want a value. It is
`match` with less ceremony, and the compiler forces the two arms to agree, so
the result type is unambiguous. `either_with` passes a context value into
whichever closure runs, which matters when both closures want to capture the
same `&mut` — only one will run, but the borrow checker cannot see that.

#### `into_inner`

When both sides are the same type, takes the value out.

```
use either::{Either, IntoEither, Left};

let x: Either<u32, u32> = Left(5);
assert_eq!(x.into_inner(), 5);

// IntoEither builds one from a bool or a predicate.
assert_eq!(7.into_either(true), Left(7));
assert_eq!(8.into_either_with(|n| n % 2 == 0).into_inner(), 8);
```

**When to use it:** at the end of a pipeline that used `Either<T, T>` to track
which path a value took, once the distinction has stopped mattering. `map` on
the same type applies one closure to whichever side is present, which is the
other half of the pattern.

#### `left` and `right`

Converts to `Option`, discarding the other side.

```
use either::{Either, Left, Right};

let a: Either<u32, &str> = Left(3);
let b: Either<u32, &str> = Right("no");

assert_eq!(a.left(), Some(3));
assert_eq!(b.left(), None);
assert_eq!(b.right(), Some("no"));
```

**When to use it:** when only one side interests you and the other is a
non-event. `left_or`, `left_or_default` and `left_or_else` go straight to a
value with a fallback — the familiar `Option` vocabulary, so there is nothing
new to learn. `unwrap_left` and `expect_left` panic instead, and like all
unwrapping should be reserved for cases a comment can justify.

### Transforming in place

#### `map_left`, `map_right` and `map_either`

Changes one side, or both, leaving which-side-it-is alone.

```
use either::{Either, Left, Right};

let n: Either<u32, &str> = Left(21);
assert_eq!(n.map_left(|v| v * 2), Left(42));

// map_either changes both sides in one call.
let s: Either<u32, &str> = Right("ok");
let normalised: Either<String, usize> =
    s.map_either(|v| v.to_string(), |v| v.len());
assert_eq!(normalised, Right(2));
```

**When to use it:** adapting the payload without deciding anything — converting
units, wrapping in a newtype, normalising both arms towards a common shape
before a later `either`. `map_left_or` maps *and* collapses, returning a default
when the value is on the other side.

#### `as_ref` and `as_mut`

Borrows the inside without consuming the `Either`.

```
use either::{Either, Left};

let mut value: Either<String, u32> = Left("text".to_string());

// Either<&String, &u32>, so the original stays put.
assert_eq!(value.as_ref().left().map(|s| s.len()), Some(4));

if let Left(s) = value.as_mut() {
    s.push('!');
}
assert_eq!(value, Left("text!".to_string()));
```

**When to use it:** inspecting or mutating a long-lived `Either` — a field on a
struct, something behind a `&self`. It is the same `as_ref`/`as_mut` pair
`Option` has, and `as_pin_ref`/`as_pin_mut` cover the pinned case, which is what
makes the `Future` impl work.

#### `flip`

Swaps the two sides.

```
use either::{Either, Left, Right};

let x: Either<u32, &str> = Left(1);
assert_eq!(x.flip(), Right(1));
```

**When to use it:** reconciling two functions that disagree about which side
means what — and that is its only real use. If you find yourself flipping often,
the convention is wrong somewhere and worth fixing instead.

### Iterating

#### `into_iter` on an `Either` of collections

When both sides iterate to the same item type, the `Either` becomes the
iterator.

```
use either::{Either, Left, Right};

let left: Either<Vec<u32>, std::ops::Range<u32>> = Left(vec![9, 8]);
let right: Either<Vec<u32>, std::ops::Range<u32>> = Right(0..3);

assert_eq!(left.into_iter().collect::<Vec<_>>(), vec![9, 8]);
assert_eq!(right.into_iter().sum::<u32>(), 3);
```

**When to use it:** holding one of two collections and wanting to walk it
without caring which. `iter` and `iter_mut` do the same by borrow. The
constraint is that the item types must match — `R: IntoIterator<Item = L::Item>`
— because the resulting iterator has to name one `Item`.

#### `factor_into_iter`

Iterates when the two sides yield *different* item types.

```
use either::{Either, Left, Right};

let words: Either<Vec<&str>, Vec<u32>> = Right(vec![1, 2]);

// Each item is itself an Either, so the types need not agree.
let shown: Vec<String> = words
    .factor_into_iter()
    .map(|item| item.either(|s: &str| s.to_string(), |n: u32| n.to_string()))
    .collect();

assert_eq!(shown, vec!["1".to_string(), "2".to_string()]);
```

**When to use it:** when `into_iter` is rejected because the item types differ.
It pushes the `Either` down one level — from "either of two iterators" to "an
iterator of eithers" — so you resolve the branch per item instead of once.
`factor_iter` and `factor_iter_mut` are the borrowing versions.

### Relating it to `Result` and `Option`

#### `From<Result<R, L>>`

Converts in both directions, with `Err` on the left.

```
use either::{Either, Left, Right};

let ok: Result<u32, String> = Ok(1);
let err: Result<u32, String> = Err("bad".to_string());

// Ok -> Right, Err -> Left.
assert_eq!(Either::from(ok), Right(1));
assert_eq!(Either::from(err), Left("bad".to_string()));

// And back again.
let back: Result<u32, String> = Right(2).into();
assert_eq!(back, Ok(2));
```

**When to use it:** at a boundary — feeding a `Result` into code that speaks
`Either`, or handing one back. Note which way round it goes: `Left` is the error
side, following the convention that the right side is the "right" answer. Do not
use `Either` as your error type; `Result` has `?`, the `Error` trait and the
whole ecosystem behind it.

#### `factor_none`, `factor_err` and `factor_ok`

Pulls a shared `Option` or `Result` wrapper out from inside both sides.

```
use either::{Either, Left, Right};

// Either<Option<A>, Option<B>> -> Option<Either<A, B>>
let present: Either<Option<u32>, Option<&str>> = Left(Some(4));
assert_eq!(present.factor_none(), Some(Left(4)));

let missing: Either<Option<u32>, Option<&str>> = Right(None);
assert_eq!(missing.factor_none(), None);

// Either<Result<A, E>, Result<B, E>> -> Result<Either<A, B>, E>
let ok: Either<Result<u32, String>, Result<&str, String>> = Right(Ok("yes"));
assert_eq!(ok.factor_err(), Ok(Right("yes")));
```

**When to use it:** after a `map_either` whose closures were both fallible, so
you are holding the wrapper on the inside and want it on the outside where `?`
can reach it. `factor_first` and `factor_second` do the same for a tuple's
shared element.

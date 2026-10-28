---
title: "proptest"
version: "1.12.0"
publisher: "Mazdak Farrokhzad (Centril), Jason Lingle (AltSysrq), publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-28"
summary: "Property-based testing: describe the shape of valid inputs, and it generates hundreds of them looking for a counterexample — then shrinks the failure to the smallest input that still breaks."
domain: "Testing & benchmarking"
categories: ["development-tools::testing"]
repository: "https://github.com/proptest-rs/proptest"
docs: "https://docs.rs/proptest/latest/proptest/"
---

## Overview

An ordinary test asserts something about inputs you chose. A property test
asserts something about *all* inputs of a shape, and lets the machine look for
the one you would never have thought of.

```
use proptest::prelude::*;

fn reverse<T: Clone>(items: &[T]) -> Vec<T> {
    items.iter().rev().cloned().collect()
}

proptest! {
    #[test]
    fn reversing_twice_restores_the_original(items: Vec<i32>) {
        prop_assert_eq!(reverse(&reverse(&items)), items);
    }
}
```

That runs 256 times by default with different vectors — empty, one element,
thousands, extreme values — and you wrote one assertion. The parameter type is
enough when it implements `Arbitrary`, which most standard types do; otherwise
you name a *strategy*.

**Shrinking is the feature, not the generation.** Random testing is easy;
reporting the failure usefully is not. When a property fails, proptest keeps
simplifying the input while the failure persists, so instead of

> failed with `[-274891, 0, 9223372036854775807, -1, 48, …]`

you get told it fails with `[0, 0]`. That difference is the whole value
proposition — the minimal case usually makes the bug obvious, and a sprawling
random one rarely does.

**Failures are remembered.** On a failure proptest writes the seed to a file
under `proptest-regressions/`, and re-runs that case first on every subsequent
run. Those files are meant to be **committed**: they turn a one-off random
discovery into a permanent regression test. Deleting them silently loses that.

**Use `prop_assert!`, not `assert!`.** The `prop_` macros return a failure
rather than panicking, which lets the shrinker keep working and lets proptest
attribute the failure to the input. A plain `assert!` works but reports worse,
and `panic!` inside a property is indistinguishable from the code under test
crashing.

**What it is good for, and what it is not.** Properties are easy to find for
anything with an inverse (encode/decode, parse/format), an invariant (a sorted
list stays sorted, a balance never goes negative), or a reference
implementation to compare against. They are hard to find for business logic
with no structure, where you end up reimplementing the function in the test —
at which point an example-based test is better. Property tests complement
example tests; they do not replace them.

Against the alternative: `quickcheck` is the older, simpler crate and still
works, but proptest's shrinking is integrated with its strategies rather than
bolted on, which makes it both better at minimising and able to shrink types
`quickcheck` cannot. Proptest is the one to start with.

Note the **MSRV is 1.88**, high for a dev-dependency. `no_std` is supported by
turning off the default `std` feature.

## When to use it

### Use case: Round-tripping a parser against its formatter

The easiest property to find, and the most productive.

```
use proptest::prelude::*;

fn format_pair(a: i32, b: i32) -> String {
    format!("{a},{b}")
}

fn parse_pair(text: &str) -> Option<(i32, i32)> {
    let (left, right) = text.split_once(',')?;
    Some((left.trim().parse().ok()?, right.trim().parse().ok()?))
}

proptest! {
    #[test]
    fn parse_undoes_format(a: i32, b: i32) {
        let formatted = format_pair(a, b);
        prop_assert_eq!(parse_pair(&formatted), Some((a, b)));
    }
}
```

**Why it fits:** you do not have to know what the tricky inputs are — negative
numbers, `i32::MIN`, zero — because the generator will find them. This property
catches a whole class of bug that example tests miss by construction: anything
you did not think to write down. Encode/decode, serialise/deserialise and
compress/decompress all have this shape, and it is the first thing to reach for
on any of them.

### Use case: Checking an invariant holds for every input

When there is no inverse, there is usually still something always true.

```
use proptest::prelude::*;

fn clamp_to_range(value: i32, low: i32, high: i32) -> i32 {
    value.max(low).min(high)
}

proptest! {
    #[test]
    fn clamp_result_is_always_within_bounds(
        value: i32,
        // Generate an ordered pair, so low <= high by construction.
        (low, high) in (any::<i32>(), any::<i32>())
            .prop_map(|(a, b)| if a <= b { (a, b) } else { (b, a) }),
    ) {
        let result = clamp_to_range(value, low, high);
        prop_assert!(result >= low);
        prop_assert!(result <= high);
    }
}
```

**Why it fits:** the invariant is the specification — a clamp that returns
something outside its bounds is wrong whatever else it does. Note the
`prop_map` building an ordered pair rather than generating two integers and
filtering: **constructing valid input beats rejecting invalid input**, because
rejection throws away generated cases and proptest will eventually give up if
too many are discarded.

### Use case: Comparing against a slow, obviously-correct implementation

Differential testing, where the property is "these agree".

```
use proptest::prelude::*;

// The fast one, which might be wrong.
fn count_evens(values: &[u32]) -> usize {
    values.iter().filter(|v| *v % 2 == 0).count()
}

// The obvious one, which is not in the hot path.
fn count_evens_naive(values: &[u32]) -> usize {
    let mut total = 0;
    for v in values {
        if v % 2 == 0 {
            total += 1;
        }
    }
    total
}

proptest! {
    #[test]
    fn optimised_matches_naive(values in prop::collection::vec(any::<u32>(), 0..100)) {
        prop_assert_eq!(count_evens(&values), count_evens_naive(&values));
    }
}
```

**Why it fits:** this is the strongest use of property testing, because the
property needs no cleverness — you already have a correct implementation, and
you are asserting the optimised one agrees. It is how you safely replace a
readable function with a fast one, and how you test a SIMD path against a scalar
one. The naive version lives in the test, so its slowness costs nothing in
production.

## API map

A macro, a library of strategies, and a configuration struct.

### Writing properties

#### `proptest!`

The macro that turns a function into a property test.

```
use proptest::prelude::*;

proptest! {
    // Typed parameters use the Arbitrary impl for that type.
    #[test]
    fn addition_commutes(a: i32, b: i32) {
        prop_assert_eq!(a.wrapping_add(b), b.wrapping_add(a));
    }

    // `in` names a strategy explicitly.
    #[test]
    fn small_numbers_stay_small(n in 0u32..100) {
        prop_assert!(n < 100);
    }
}
```

**When to use it:** for every property. The macro generates a real `#[test]`
function, so `cargo test` finds it and filters by name as usual. Prefer the
typed form where an `Arbitrary` impl exists; reach for `in` when you need to
constrain the range, which in practice is often — unbounded `Vec<T>` generation
can be slower than you want.

#### `prop_assert!` and friends

Assertions that cooperate with shrinking.

```
use proptest::prelude::*;

proptest! {
    #[test]
    fn assertions(a in 1i32..100, b in 1i32..100) {
        // Returns a failure instead of panicking.
        prop_assert!(a + b > a);
        prop_assert_eq!(a + b, b + a);
        prop_assert_ne!(a + b, 0);

        // With a message, as with the std macros.
        prop_assert!(a > 0, "a was {}", a);
    }
}
```

**When to use it:** always, inside a property. `prop_assume!` is the related one
worth knowing: it discards the current case rather than failing it, for a
precondition you could not express in the strategy — though a strategy that
only generates valid input is better, since too many discards abort the run.

#### `ProptestConfig`

How many cases, and how hard to try.

```
use proptest::prelude::*;

proptest! {
    // An inner attribute, and it must come first in the block.
    #![proptest_config(ProptestConfig {
        cases: 1000,
        max_shrink_iters: 10_000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn runs_a_thousand_times(n in 0u64..1_000_000) {
        prop_assert!(n < 1_000_000);
    }
}
```

**When to use it:** raise `cases` for a cheap property worth hammering, lower it
for an expensive one — the default is 256. `max_shrink_iters` matters when
shrinking a large structure takes too long and you would rather have a
non-minimal counterexample than wait. Note the placement: `#![proptest_config]`
is an inner attribute and **must be the first thing in the block**, before any
test, or you get a confusing parse error from inside the macro.

### Strategies

#### Primitives and ranges

The building blocks.

```
use proptest::prelude::*;

proptest! {
    #[test]
    fn ranges(
        // Any value of the type.
        anything in any::<u8>(),
        // A bounded range.
        bounded in 10i64..=20,
        // A choice from a fixed set.
        chosen in prop::sample::select(vec!["a", "b", "c"]),
        // A string matching a regex.
        word in "[a-z]{3,8}",
    ) {
        prop_assert!(bounded >= 10 && bounded <= 20);
        prop_assert!(["a", "b", "c"].contains(&chosen));
        prop_assert!((3..=8).contains(&word.len()));
        let _ = anything;
    }
}
```

**When to use it:** ranges constantly, since unbounded generation is rarely what
you mean. The regex strategy is the one people are surprised by — a `&str`
literal *is* a strategy producing strings matching it, which is how you generate
plausible identifiers, dates or codes without writing a generator. `select`
picks from a list, with shrinking toward the first element.

#### Collections

Vectors, maps and sets.

```
use proptest::prelude::*;

proptest! {
    #[test]
    fn collections(
        // Length range is required, which is deliberate.
        items in prop::collection::vec(any::<i32>(), 0..50),
        pairs in prop::collection::hash_map("[a-z]{1,4}", any::<u32>(), 1..10),
        set in prop::collection::btree_set(0u8..100, 0..20),
    ) {
        prop_assert!(items.len() < 50);
        prop_assert!(!pairs.is_empty());
        prop_assert!(set.len() <= 20);
    }
}
```

**When to use it:** whenever the input is a collection, which is most of the
time. The size range is not optional, and that is the point — an unbounded
`Vec<Vec<String>>` will generate something enormous and your test will appear to
hang. Keep the bounds small: a bug that needs fifty elements to show up is rare,
and a hundred runs of a fifty-element case is quick.

#### `prop_map`, `prop_filter` and `prop_flat_map`

Deriving one strategy from another.

```
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,
    age: u8,
}

proptest! {
    #[test]
    fn mapped(
        // Build a valid value from simpler parts.
        user in ("[a-z]{1,10}", 18u8..100).prop_map(|(name, age)| User { name, age }),
        // Filter, when construction is not possible.
        even in any::<u32>().prop_filter("must be even", |n| n % 2 == 0),
        // flat_map: the second strategy depends on the first value.
        (len, items) in (1usize..10)
            .prop_flat_map(|len| (Just(len), prop::collection::vec(any::<u8>(), len))),
    ) {
        prop_assert!(user.age >= 18);
        prop_assert_eq!(even % 2, 0);
        prop_assert_eq!(items.len(), len);
    }
}
```

**When to use it:** `prop_map` for building a domain type out of primitives,
which is how you get meaningful generated data rather than arbitrary bytes.
`prop_flat_map` when one part of the input constrains another — a length and a
vector of that length, an index into a collection you just generated. Use
`prop_filter` sparingly: it rejects cases, and proptest aborts the run if the
rejection rate gets too high. `Just` wraps a fixed value as a strategy.

### Running without the macro

#### `TestRunner`

Driving proptest from ordinary code.

```
use proptest::prelude::*;
use proptest::test_runner::TestRunner;

#[test]
fn run_manually() {
    let mut runner = TestRunner::default();

    runner
        .run(&(0i32..100), |value| {
            prop_assert!(value < 100);
            Ok(())
        })
        .unwrap();
}
```

**When to use it:** when the property cannot be a top-level test — inside a
loop over configurations, in a custom harness, or when you need to inspect the
outcome rather than fail the test. The closure returns `TestCaseResult`, so
`Ok(())` for a pass and the `prop_assert!` macros for a failure. Most of the
time `proptest!` is what you want; this is the escape hatch.

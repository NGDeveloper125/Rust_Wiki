---
title: ".. / ..= / ..."
kind: punctuation
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching"]
related_concepts: ["Destructuring", "Slices", "The Iterator trait"]
related_syntax: ["[ ]", "@", "_"]
see_also: ["[ ]", "@"]
---

## Explanation

Three related tokens, disambiguated by position, cover ranges in Rust:

**`..` — rest-of-pattern.** Inside a struct, tuple, or slice pattern, `..`
stands for "the remaining fields/elements, which this pattern doesn't name."
`Point { x, .. }` matches any `Point` and binds only `x`, ignoring every
other field; `[first, ..]` matches a slice of any length ≥ 1 and binds only
its first element. A name placed before `@` and after `..` inside a slice
pattern (`[first, rest @ ..]`) binds the ignored middle as a subslice
instead of discarding it entirely. See
[destructuring](../../concepts/pattern-matching/destructuring.md) for the
broader pattern-matching context this fits into.

**`..` — exclusive range expression.** Outside pattern position, `a..b`
builds a `Range` value covering `a` up to but not including `b`, used most
often to drive a `for` loop or as an index into [`[ ]`](square-brackets.md)
for slicing. The endpoints are optional: `a..` (`RangeFrom`, unbounded
above), `..b` (`RangeTo`, unbounded below), and `..` alone (`RangeFull`,
the whole collection, as in `&data[..]`).

**`..=` — inclusive range.** `a..=b` includes `b` itself, both as an
expression (`RangeInclusive`) and, unlike bare `..`, also as a *pattern*:
`1..=5 => ...` in a `match` arm matches any value from `1` through `5`
inclusive. Plain `..` is not legal in pattern position for a bounded range
— only `..=` is.

**`...` — deprecated inclusive range pattern.** Before `..=` was stabilized
(Rust 1.26), inclusive range *patterns* were written `1...5`. That syntax
is obsolete: it has no expression-position meaning at all (there was never
a `1...5` range expression), and in pattern position current Rust rejects
it outright rather than merely warning. There is no reason to write it in
new code — `..=` is its direct, and only, replacement.

## Basic usage example

```
for i in 0..5 { // <- `..` exclusive range: yields 0, 1, 2, 3, 4
    println!("{i}");
}
```

## Best practices & deeper information

### Scenario: Working with collections

Averaging a fixed-size window of recent sensor readings needs a slice of
exactly that window — an exclusive range expression is the index that
picks it out.

```
fn moving_average(readings: &[f64], start: usize, window: usize) -> f64 {
    let end = start + window;
    let slice = &readings[start..end]; // <- `..` as an exclusive range used to index a slice
    slice.iter().sum::<f64>() / slice.len() as f64
}

let readings = [21.0, 21.4, 21.6, 22.1, 21.9];
println!("{:.2}", moving_average(&readings, 1, 3));
```

**Why this way:** `start..end` reads as "from `start`, up to but not
including `end`," matching how `slice.len()` and index arithmetic already
work in Rust — the
[std docs on slice indexing](https://doc.rust-lang.org/std/primitive.slice.html)
use this same half-open convention throughout the standard library.

### Scenario: Branching on data (pattern matching)

Summarizing a list of scores needs different handling for an empty list, a
single score, and everything else — `..` in a slice pattern matches "the
rest," however many elements that turns out to be.

```
fn summarize(scores: &[u32]) -> String {
    match scores {
        [] => "no scores".to_string(),
        [only] => format!("single score: {only}"),
        [first, ..] => {
            // <- `..` matches the remaining elements without naming or counting them
            format!("first score: {first}, {} more", scores.len() - 1)
        }
    }
}

println!("{}", summarize(&[92, 87, 95]));
```

**Why this way:** a slice pattern with `..` is exhaustive over "any
length ≥ 1" in one arm, instead of writing out a separate arm per possible
length — the
[Rust Reference on slice patterns](https://doc.rust-lang.org/reference/patterns.html#slice-patterns)
documents `..` as matching zero or more elements at that position.

### Scenario: Numeric computation

Assigning a letter grade from a numeric score is a direct fit for
inclusive range patterns, since a grade boundary (100) belongs to the band
below it, not the one above.

```
fn letter_grade(score: u8) -> char {
    match score {
        90..=100 => 'A', // <- `..=`: 100 is included in this arm
        80..=89 => 'B',
        70..=79 => 'C',
        _ => 'F',
    }
}

println!("{}", letter_grade(90));
```

**Why this way:** writing `90..=100` states the boundary directly, where a
bare `90..101` (or two overlapping-looking `90..100` arms) would leave a
reader to double-check whether the endpoint is included — the
[Rust Reference on range patterns](https://doc.rust-lang.org/reference/patterns.html#range-patterns)
requires `..=`, not `..`, for exactly this inclusive case.

## Embedded Rust Notes

**Full support.** `Range`/`RangeInclusive` and range patterns are
core-language and allocator-free; range-driven iteration compiles down to a
simple counter and comparison. Common for walking a fixed set of register
indices or classifying a raw ADC value into a named band via range
patterns, with no runtime cost beyond the comparisons themselves.

---
title: "enum"
kind: keyword
embedded_support: full
groups: ["Types & Data Structures", Basics]
related_concepts: ["Enums (algebraic data types)"]
related_syntax: [struct, match, "#[repr(...)]"]
see_also: [struct]
---

## Explanation

`enum` declares a new sum type: a name followed by a brace-delimited list
of variants, each variant itself using one of the same three field
grammars a `struct` can use. A **unit-like** variant is just a bare name
(`Active`); a **tuple-like** variant carries positional fields in
parentheses (`Paused(u32)`); a **struct-like** variant carries named
fields in braces (`Stopped { reason: String }`). A single `enum` is free
to mix all three variant kinds, separated by commas, with an optional
trailing comma before the closing brace — no semicolon follows the block,
same as a named-field struct.

A variant is referred to as `EnumName::VariantName`, and a struct-like or
tuple-like variant is constructed or destructured exactly the way its
matching struct form would be (`Stopped { reason }`, `Paused(secs)`).
Like `struct`, `enum` accepts generic parameters and a `where` clause
right after the name: `enum Either<L, R> { Left(L), Right(R) }`.

Unit-like variants — and only unit-like variants, unless every variant in
the enum is unit-like — may carry an **explicit discriminant**: `enum
Priority { Low = 1, Medium = 5, High = 10 }`. Without an explicit value, a
variant's discriminant is the previous variant's plus one, starting from
`0`. Discriminants are otherwise an implementation detail the compiler is
free to choose; pinning them to a specific integer width for FFI or a
wire format is what [`#[repr(u8)]` and
friends](../attributes/repr.md) are for, layered on top of the `enum`
declaration itself.

Choosing what an enum's variants should look like, and why a sum type
fits a given problem, is design territory covered on the
[Enums (algebraic data types)](../../concepts/types-data-modeling/enums-algebraic-data-types.md)
concept page; this page covers only the declaration grammar.

## Basic usage example

```
enum Status {
    Active,                    // <- unit-like variant: no data
    Paused(u32),                // <- tuple-like variant: positional data
    Stopped { reason: String }, // <- struct-like variant: named data
}

let s = Status::Paused(30); // <- `enum` values are always one specific variant
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

Each variant kind is destructured in a `match` arm with the same grammar
it was declared with — unit-like variants need no parentheses or braces
at all, which is part of what makes matching read as plainly as the
declaration itself.

```
enum Shape {
    Point,                              // <- unit-like
    Circle(f64),                        // <- tuple-like
    Rectangle { width: f64, height: f64 }, // <- struct-like
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Point => 0.0,                                   // <- no fields to bind
        Shape::Circle(radius) => std::f64::consts::PI * radius * radius, // <- positional binding
        Shape::Rectangle { width, height } => width * height,  // <- named binding
    }
}
```

**Why this way:** matching each variant with its own declared shape means
there's exactly one way to write the pattern for a given variant, so a
reader who knows the `enum` declaration already knows how every `match`
arm on it will look — the
[Rust Book](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)
introduces all three variant forms together for this reason.

### Scenario: Designing a public API

Marking a public, growth-prone enum `#[non_exhaustive]` is written as an
attribute directly above the `enum` item, not as part of the variant list
itself — it changes what downstream `match` expressions on that enum are
required to include.

```
#[non_exhaustive] // <- attribute sits above the `enum` keyword, not inside the variant list
pub enum PaymentMethod {
    Card,
    BankTransfer,
}
```

**Why this way:** `#[non_exhaustive]` is the attribute-level tool for
enums a library expects to grow — see
[`#[non_exhaustive]`](../attributes/non-exhaustive.md) for the full
grammar and its effect on downstream `match`es, which the
[API Guidelines](https://rust-lang.github.io/api-guidelines/future-proofing.html)
recommend for exactly this kind of public, extensible enum.

## Embedded Rust Notes

**Full support.** Enum declarations are core-language and
allocator-free — their size is roughly the largest variant plus, when
needed, a discriminant (niche optimization often folds the tag into a
variant's unused bit patterns, so `Option<&T>` stays pointer-sized).
Explicit discriminants combined with `#[repr(u8)]` are common for
decoding register states and protocol message kinds in embedded code.

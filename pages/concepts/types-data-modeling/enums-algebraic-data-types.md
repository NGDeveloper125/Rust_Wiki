---
title: "Enums (algebraic data types)"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Functional Programming", "Designing Robust Data Models", "State Machines", "Recursive Data Structures", "Coming from Java / C#", "Coming from Haskell / functional languages"]
related_syntax: [enum]
see_also: ["match expressions", "Exhaustiveness checking", "\"Make invalid states unrepresentable\""]
---

## Explanation

An enum defines a type as one of several distinct variants, each of which
can optionally carry its own data:

```
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Triangle { base: f64, height: f64 },
}
```

This is what's meant by "algebraic data type" — a *sum* type, where a
value is exactly one of several alternatives (as opposed to a struct, a
*product* type, where a value is all of its fields at once). This is
categorically more powerful than the "enum" in languages like C, Java, or
C# (before their more recent additions), where an enum is just a named
set of integer-like constants — a Rust enum variant can carry arbitrary,
variant-specific data, which is what makes `Option<T>` and `Result<T, E>`
possible as ordinary enums rather than special-cased language features.

Combined with [`match`](../pattern-matching/match-expressions.md), enums
are the primary tool for making illegal states genuinely unrepresentable:
a value can only ever be one of the variants you defined, each variant
can only carry exactly the data appropriate to that case, and the
compiler forces every `match` to handle every variant (see
[Exhaustiveness checking](../pattern-matching/exhaustiveness-checking.md)) —
so adding a new variant later surfaces every place in the codebase that
needs updating, as a compile error, rather than a silent gap in behavior.

## Embedded Rust Notes

**Full support.** Enums are core-language and allocator-free (their size
is the max of their variants plus a discriminant, computed at compile
time) — ideal for representing peripheral states, register field values,
or protocol message types with zero runtime allocation.

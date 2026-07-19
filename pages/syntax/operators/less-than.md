---
title: "<"
kind: operator
embedded_support: full
groups: [Basics, "Types & Data Structures"]
related_concepts: [Operator overloading, Generics]
related_syntax: [">", "<=", ">="]
see_also: [">"]
---

## Explanation

`<` is the less-than comparison, overloadable via `std::ops::PartialOrd`:

```
if a < b { ... }
```

`<` is also the opening delimiter for **generic parameter lists**
(`Vec<T>`, `fn f<T>()`) — an entirely different, non-operator role. This
dual use is why the parser sometimes needs help disambiguating generics
from a chained comparison (`a < b, c > d` reads ambiguously); the
"turbofish" `::<...>` exists specifically to disambiguate generics in
expression position (see [`::`](path-separator.md)).

## Embedded Rust Notes

**Full support.** `PartialOrd` lives in `core::cmp`; generics/turbofish
are pure compile-time grammar. No `std` dependency either way.

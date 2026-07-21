---
title: "type"
kind: keyword
embedded_support: full
groups: ["Types & Data Structures", Basics]
related_concepts: ["Type aliases", "Associated types"]
related_syntax: [struct, trait]
see_also: []
---

## Explanation

`type` has two distinct declaration forms, depending on where it appears.

At module scope, inside a `fn` body, or inside an `impl` block, `type
Name = ExistingType;` declares a **type alias**: a new name for an
already-existing type, with an `=` and a terminating `;`. The alias may
take its own generic parameters, most commonly to shorten a crate's own
`Result`: `type Result<T> = std::result::Result<T, MyError>;`. Wherever
this form appears, the right-hand side must be a complete, already-known
type — there's no way to leave it unresolved.

Inside a `trait` body, the grammar is different: `type Item;` — no `=`,
just a name and a semicolon — declares an **associated type**: a type
slot every implementer of the trait must fill in. It may carry its own
trait bound, exactly like a generic parameter would (`type Item:
std::fmt::Display;`), constraining what any implementer is allowed to set
it to, without fixing the type itself. Inside an `impl` block for that
trait, the same `type Item = ConcreteType;` syntax as a plain alias fills
the slot in — `=` and all — but here it's binding, once, the type that
specific implementation uses, not merely renaming something for
readability.

The two forms are easy to conflate because the implementing side looks
identical to a plain alias declaration; the distinguishing signal is
always the trait declaration itself: a bodyless `type Name;` (optionally
bounded) inside a `trait` block is always an associated-type slot, never
an alias.

The reasoning behind reaching for an alias (pure readability, zero type
safety) versus an associated type (exactly one implementer-chosen type
per trait implementation) is covered on the
[Type aliases](../../concepts/types-data-modeling/type-aliases.md) and
[Associated types](../../concepts/types-data-modeling/associated-types.md)
concept pages; this page covers only the two declaration grammars.

## Basic usage example

```
type Kilometers = f64; // <- alias form: `=` and `;`, no trait involved

trait Container {
    type Item; // <- associated-type form: no `=`, just a name inside a trait
    fn get(&self) -> Self::Item;
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A crate's own `Result` alias, generic over just the success type, keeps
every fallible function's signature from having to spell out the
concrete error type by hand.

```
#[derive(Debug)]
pub struct ConfigError(String);

pub type Result<T> = std::result::Result<T, ConfigError>; // <- `type` here is a plain alias, generic over T

pub fn load_port(raw: &str) -> Result<u16> { // <- reads far better than Result<u16, ConfigError> repeated everywhere
    raw.parse().map_err(|_| ConfigError("invalid port".into()))
}
```

**Why this way:** this is one of the most common uses of the alias form
of `type` in real crates — every public fallible function shares one
`Result<T>` alias instead of repeating the crate's error type at every
call site, a pattern the
[std library itself uses](https://doc.rust-lang.org/std/io/type.Result.html)
for `std::io::Result<T>`.

### Scenario: Writing generic code

A trait whose method has exactly one correct return type per
implementer declares that type as an associated type with `type`, rather
than as a generic parameter the caller would otherwise have to specify at
every call site.

```
trait Parser {
    type Output; // <- `type` with no `=`: each implementer must fill this in exactly once
    fn parse(&self, input: &str) -> Self::Output;
}

struct IntParser;
impl Parser for IntParser {
    type Output = i32; // <- `type` with `=`, inside the impl: binds Output to i32 for this implementer
    fn parse(&self, input: &str) -> i32 {
        input.parse().unwrap_or(0)
    }
}
```

**Why this way:** writing `Self::Output` in the trait's signature instead
of a second generic parameter keeps `fn parse(&self, input: &str) ->
Self::Output` free of a type parameter that was never the caller's to
choose — see
[Associated types](../../concepts/types-data-modeling/associated-types.md)
for when this is preferable to a generic parameter instead.

## Embedded Rust Notes

**Full support.** Both forms are purely compile-time — no `std` or
allocator dependency. The `embedded-hal` ecosystem's traits rely heavily
on associated types (an error type tied to a specific peripheral trait,
for instance) to stay generic across vendors' hardware without a
proliferation of generic parameters.

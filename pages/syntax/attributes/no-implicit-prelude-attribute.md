---
title: "#[no_implicit_prelude]"
kind: attribute
embedded_support: full
groups: ["Memory & Unsafe"]
related_concepts: []
related_syntax: [mod, use]
see_also: []
---

## Explanation

`#[no_implicit_prelude]` is placed on a `mod` item (or as `#![no_implicit_prelude]`
at the crate root, covering every module) and stops the compiler from
automatically bringing the standard prelude — or, in a `#![no_std]`
crate, the `core` prelude — into scope inside that module. It also
disables the implicit injection of the current crate's own prelude, if
one is configured via `Cargo.toml`. Everything that would otherwise be
silently available (`Option`, `Some`, `None`, `Vec`, `String`, `Box`,
`Copy`, `Clone`, and the rest of the prelude's items) must be brought in
with an explicit `use` instead, or referred to by its fully-qualified
path.

This is a narrow, special-purpose attribute. Ordinary application and
library code essentially never reaches for it — losing implicit access to
`Option`/`Result`/`Vec` for no benefit would just be friction. Its real
audience is code-generation tooling: a proc-macro or build-script-driven
codegen tool emitting a module of programmatically generated identifiers
sometimes wants a **guaranteed empty namespace**, so that a generated
name can never silently collide with, or be shadowed by, a prelude item
the generator's author didn't anticipate. With
`#[no_implicit_prelude]`, the generated module starts from a blank slate
and the generator writes out every `use` it actually needs, making name
collisions a compile error at the point of generation rather than a
subtle runtime surprise.

`#[no_implicit_prelude]` has no effect on the *contents* of the module
beyond scope resolution — the items already written inside are unchanged,
only what's implicitly visible to them.

## Basic usage example

```
#[no_implicit_prelude] // <- nothing from the standard prelude is implicitly in scope here
mod generated {
    pub fn identity(x: i32) -> i32 {
        x // no `Option`/`Vec`/etc. needed here, so nothing is missing
    }
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A schema-to-Rust codegen tool emits a module of generated field-accessor
functions and wants an absolute guarantee that none of its generated
identifiers can accidentally resolve to a prelude item instead of the
name the generator intended.

```
#[no_implicit_prelude] // <- guarantees a blank namespace: no accidental prelude shadowing
mod schema_generated {
    use core::option::Option; // <- must be spelled out explicitly; nothing is implicit here

    pub struct Record {
        pub id: u64,
        pub label: Option<&'static str>,
    }
}
```

**Why this way:** codegen tools produce identifiers programmatically and
can't always predict every name a future prelude addition might
introduce; starting from a namespace with nothing implicit removes that
entire class of surprise collision, which the
[Rust Reference](https://doc.rust-lang.org/reference/names/preludes.html#the-no_implicit_prelude-attribute)
documents as the attribute's intended use — explicit, generated `use`
statements for every name the module actually needs, rather than relying
on whatever the prelude happens to contain.

## Embedded Rust Notes

**Full support.** `#[no_implicit_prelude]` is a name-resolution-only,
compile-time attribute with no runtime behavior and no dependency on
`alloc`/`std` — it behaves identically in a `#![no_std]` crate, disabling
injection of the `core` prelude instead of the `std` one. Its use case
(codegen tooling wanting a guaranteed-empty namespace) is no more or less
common in embedded Rust than anywhere else.

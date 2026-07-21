---
title: "#[no_implicit_prelude]"
kind: attribute
embedded_support: full
groups: ["No-std & Embedded Runtime", "Memory & Unsafe"]
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

## Usage examples

### Blocking implicit prelude imports in a module

```
#[no_implicit_prelude] // <- nothing from the standard prelude is implicitly in scope here
mod generated {
    pub fn identity(x: i32) -> i32 {
        x // no `Option`/`Vec`/etc. needed here, so nothing is missing
    }
}
```

### Designing a public API

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

Codegen tools produce identifiers programmatically and
can't always predict every name a future prelude addition might
introduce; starting from a namespace with nothing implicit removes that
entire class of surprise collision, which the
[Rust Reference](https://doc.rust-lang.org/reference/names/preludes.html#the-no_implicit_prelude-attribute)
documents as the attribute's intended use — explicit, generated `use`
statements for every name the module actually needs, rather than relying
on whatever the prelude happens to contain.

## Explanation (Embedded)

Honestly, there isn't a distinct embedded story here. `#[no_implicit_prelude]`
is a name-resolution-only, compile-time attribute with no runtime
behavior and no dependency on `alloc`/`std` — it works exactly the same
way in a `#![no_std]` crate, disabling injection of the `core` prelude in
place of the `std` one, and nothing about being on bare metal changes
what it does or how it's used. Its actual audience — codegen tooling that
wants a guaranteed-empty namespace so generated identifiers can never
collide with a prelude item — does exist in embedded Rust: `svd2rust`
generates an entire peripheral-access crate's worth of register and field
types from a chip vendor's SVD file, which is exactly the shape of
large-scale, tool-generated code this attribute is meant for. But
`svd2rust`'s generated code doesn't actually reach for
`#[no_implicit_prelude]` in practice, any more than most other Rust
codegen tools do — so even here, this stays a rare attribute, not one
with genuine embedded-specific pull.

## Usage examples (Embedded)

### A peripheral-register codegen module, still not reaching for this attribute

```
#[no_implicit_prelude] // <- guarantees no accidental collision with prelude items
mod gpio_regs_generated {
    use core::marker::PhantomData; // must be spelled out; nothing implicit here

    pub struct MODER<T> {
        pub(crate) _marker: PhantomData<T>,
    }
}
```

This is the same kind of guaranteed-empty-namespace guarantee a
non-embedded codegen tool would want, just applied to a register-
definition module instead of a general one — the attribute itself
behaves identically to its non-embedded usage, and most real embedded
codegen (including `svd2rust` itself) doesn't actually opt into it.

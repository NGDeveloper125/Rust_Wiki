---
title: "Temporary mutability"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms"]
related_syntax: [mut, let]
see_also: ["Constructor functions (new() convention)", "Move semantics", "Immutability by default"]
---

## Explanation

Temporary mutability is the idiom of making a binding `mut` only for the
short window where it's actually being built or assembled, then removing
mutability the moment that work is done — either by shadowing it with a
new, immutable `let` of the same name, or by scoping the mutable work
inside a block expression whose final value becomes an immutable binding
outside it. Either way, the rest of the function only ever sees the
finished, immutable value; the fact that construction needed `mut` at
all is invisible past that point.

The motivation is the same one behind
[immutability by default](../ownership-borrowing/immutability-by-default.md)
generally: a binding that stays `mut` for its entire lifetime tells every
reader "this could change anywhere below this line," even if, in
practice, all the mutation happens in the first five lines and the value
is read-only for the following fifty. Shrinking the `mut` window to
exactly where mutation happens keeps that signal accurate — a `mut` that
appears in scope is a `mut` a reader has to actually worry about.

The shadowing form (`let mut v = ...; /* build v */ let v = v;`) is the
lighter-weight option when the mutable phase is just a few statements in
the same function; the block-expression form (`let v = { let mut tmp =
...; /* build tmp */ tmp };`) is preferable when the construction logic
is long enough to want its own visual scope, since it makes the boundary
between "building" and "built" a real syntactic block rather than a
single re-declaration easy to miss while skimming.

Neither form changes what the compiler generates — the rebinding is
purely a hint to the reader (and to lints like Clippy's `unused_mut`,
which would otherwise flag a `mut` binding that's never mutated again
after the rebinding point). The value itself doesn't move or get copied
by the shadowing; only the binding's mutability annotation changes.

## Basic usage example

```
let mut greeting = String::new(); // <- mut needed only while building the value
greeting.push_str("hello, ");
greeting.push_str("world");
let greeting = greeting; // <- shadows the binding as immutable; nothing below this line can mutate it

println!("{greeting}");
```

## Best practices & deeper information

### Scenario: Creating a new object

Building a `Config` requires several mutating steps, so the construction
work happens inside a block expression, and only the finished, read-only
value escapes it.

```
struct Config {
    entries: Vec<(String, String)>,
}

let config = { // <- block expression scopes the mutable phase
    let mut entries = Vec::new();
    entries.push(("host".to_string(), "localhost".to_string()));
    entries.push(("port".to_string(), "8080".to_string()));
    Config { entries } // last expression: becomes the outer, immutable `config`
};

// config is never `mut` from here on; nothing further in this scope can mutate it
println!("{} entries", config.entries.len());
```

**Why this way:** scoping the `mut` phase to the block keeps the mutation
window visually contained, so a reader skimming past the block already
knows `config` is finished and read-only — the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/temporary-mutability.html)
book documents this exact block-expression shape as the idiomatic way to
build a value that needs temporary mutation.

### Scenario: Working with collections

A list of sensor readings needs sorting before it's used for lookups, but
nothing after that point should be able to reorder it — shadowing the
binding immutable right after sorting makes that guarantee visible in the
code itself.

```
let mut readings = vec![19.5, 22.1, 18.0, 25.3];
readings.sort_by(|a, b| a.partial_cmp(b).unwrap()); // <- the only mutation this Vec ever needs
let readings = readings; // <- shadowed immutable: sorted order is now guaranteed not to change

println!("lowest: {}", readings[0]);
```

**Why this way:** once a collection is sorted for a purpose like binary
search or displaying a ranked list, further mutation would silently
invalidate that invariant — rebinding immutable turns "don't mutate this
again" from a comment into something the compiler enforces for the rest
of the scope.

## Embedded Rust Notes

**Full support.** Temporary mutability is a compile-time-only binding
annotation with zero runtime representation — it costs nothing and
behaves identically under `#![no_std]`, including on targets with no
allocator, since it applies equally well to a stack-allocated buffer or
fixed-size array being assembled before use.

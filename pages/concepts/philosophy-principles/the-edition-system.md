---
title: "The edition system"
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Unique to Rust"]
related_syntax: []
see_also: ["Cargo & Cargo.toml", "Dependency management & semver"]
---

## Explanation

An edition is a named, opt-in snapshot of the language's syntax and a
handful of default behaviors — Rust 2015, 2018, 2021, and 2024 so far —
declared per crate through the `edition` field in
[`Cargo.toml`](../modules-crates-visibility/cargo-and-cargo-toml.md). It
exists to solve a genuine tension: a language that never breaks anything
stagnates, but a language willing to break existing code the moment a new
compiler ships fractures its own ecosystem into incompatible islands.
Editions let Rust do the former without the latter — new syntax and
default behaviors are introduced, but only for the crates that explicitly
opt in by declaring the new edition, while everything else keeps compiling
exactly as before.

The mechanism is narrower than it might first sound, and that narrowness
is the point. An edition is resolved entirely in the compiler's front end,
before type checking begins — it changes what counts as legal syntax and a
few default behaviors (which keywords are reserved, how module paths
resolve, how closures capture their environment), but it does not fork the
type system, the standard library's ABI, or the borrow checker. A crate
declared as edition 2015 and a crate declared as edition 2024 compile with
the same compiler, link against the same `std`, and can depend on each
other freely in the same build — because edition is a per-crate setting,
not a per-compilation one. Each crate in a dependency graph is parsed
according to its own declared edition, invisibly to everything that
depends on it or that it depends on.

That constraint also defines what an edition is *not* allowed to do. Past
editions have reserved new keywords (`async`, `await`, `try`, `dyn` as a
mandatory prefix), changed default path resolution (2018's "uniform
paths"), and changed default closure-capture behavior (2021's disjoint
field captures) — always paired with an automated migration:
`cargo fix --edition` mechanically rewrites the overwhelming majority of
code affected by the change. What an edition cannot do is change a
standard library function's signature or ABI, undo a stability guarantee
already made under Rust's broader stability promise, or otherwise alter
the meaning of code in a way that would silently change the behavior of
an existing, unmodified crate that hasn't opted in. Only changes narrow
and mechanical enough to be automatically migrated are eligible at all —
which means most of what people wish they could retroactively fix (an
awkwardly named standard type, an old API's rough edges) simply isn't in
scope for this mechanism. An edition bump is a much smaller, more targeted
tool than "a major version bump" in most other languages' sense of the
phrase.

This is also a distinct axis from [semantic versioning](../modules-crates-visibility/dependency-management-and-semver.md):
semver governs a crate's own promises about its public API surface, while
the edition system governs the *language's* promises about its own
syntax. A crate can bump its edition without that being a breaking change
to its API at all, in principle — bumping the edition changes how the
crate's own source is parsed, not what it exposes to callers — though in
practice a maintainer often bundles an edition bump with other changes
that do bump the version for unrelated reasons.

New editions have shipped roughly every three years so far, each announced
well ahead of time with a migration guide and compiler lints that flag
exactly what the new edition would change before you opt in. That
deliberate infrequency and narrowness is itself the tradeoff being made:
Rust gives up "we can fix any past design mistake whenever we want" in
exchange for "the entire crates.io ecosystem keeps compiling together,
indefinitely, with no crate ever left behind by a language update it
didn't ask for."

## Basic usage example

```
[package]
name = "telemetry-agent"
version = "0.3.0"
edition = "2024" # <- opts this crate into 2024-edition syntax and defaults; every dependency keeps its own edition
```

## Best practices & deeper information

### Scenario: Designing a public API

A crate maintainer bumping from edition 2018 to 2021 needs to know which
migration is mechanical and which changes actual runtime behavior — the
2021 edition's disjoint closure captures is a real, edition-gated change
in what a closure captures, not just new syntax.

```
// Cargo.toml declares: edition = "2021" (or later) — that setting is what enables the capture below

struct Sensor { id: u32, reading: f64 }

fn log_reading(sensor: Sensor) {
    let print_id = move || println!("sensor {}", sensor.id); // <- 2021+: captures only sensor.id, not all of `sensor`
    print_id();
    println!("{}", sensor.reading); // <- compiles under 2021+: sensor.reading was never moved into the closure
}

log_reading(Sensor { id: 7, reading: 21.4 });
```

**Why this way:** under the 2018 edition and earlier, `move` captured the
whole `sensor` struct, so the second `println!` would fail to compile as
a use of a moved value; the
[Rust Edition Guide's disjoint-capture chapter](https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html)
documents this as exactly the kind of change editions exist for — a real
behavior difference, gated behind an opt-in edition bump and covered by an
automated `cargo fix --edition` migration, rather than a silent change
imposed on every crate at once.

## Explanation (Embedded)

There is honestly very little embedded-specific to say here, and it's
worth stating that plainly rather than stretching for an angle that isn't
there: the edition system is resolved entirely in the compiler's front
end, before code generation ever begins, so it has no way to know or care
whether the crate it's parsing will end up running under an OS or on a
microcontroller with no OS at all. A `#![no_std]` peripheral-access crate
or HAL declares `edition = "2021"` (or whichever) in its `Cargo.toml`
exactly the same way a hosted web service does, and every edition-gated
syntax and default-behavior change applies identically either way.

The one place edition specifics are genuinely worth knowing in an embedded
codebase is reserved syntax: register-definition and peripheral-access
code — hand-written `macro_rules!` helpers, and especially the code
`svd2rust` generates from a chip's SVD file — pastes identifiers and
literals together more aggressively than most application code, which is
exactly the pattern that can accidentally produce a 2021-reserved-prefix
or 2024-reserved-guard shape. That's covered in full, with the actual
register-macro example, on
[Reserved syntax & edition gotchas](../../syntax/punctuation/reserved-syntax-and-edition-gotchas.md)
— it applies to embedded macro-heavy code exactly as written there, with
nothing to add beyond a pointer to it.

## Basic usage example (Embedded)

```
[package]
name = "sensor-driver"
version = "0.1.0"
edition = "2024" # <- same field, same meaning, whether the crate targets an OS or bare metal

[dependencies]
cortex-m-rt = "0.7"
```

## Best practices & deeper information (Embedded)

### Scenario: Bit manipulation and flags

A register-definition macro used across a peripheral-access crate needs
to keep pasting identifiers, addresses, and bit-mask literals together
without ever producing a 2021-reserved `ident"..."` shape — the fix is the
same mechanical one that applies outside embedded code, just more likely
to come up here given how much of a PAC's source is macro-generated.

```
macro_rules! define_register {
    ($name:ident, $addr:expr) => {
        const $name: u32 = $addr; // <- kept as two separate tokens, never pasted into an ident"..." shape
    };
}

define_register!(GPIOA_BASE, 0x4001_0800);
```

**Why this way:** this is the same register-definition example covered on
[Reserved syntax & edition gotchas](../../syntax/punctuation/reserved-syntax-and-edition-gotchas.md),
included here only to make the point concrete: the edition-gated reserved
prefix rule is a lexer-level restriction that shows up in embedded code
through ordinary macro hygiene, not through anything the edition system
does differently for a bare-metal target.

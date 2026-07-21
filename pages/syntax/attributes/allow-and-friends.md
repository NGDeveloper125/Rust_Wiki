---
title: "#[allow(...)] / #[warn(...)] / #[deny(...)] / #[forbid(...)]"
kind: attribute
embedded_support: full
groups: ["Lints & Diagnostics", "Design Patterns & Idioms"]
related_concepts: ["Anti-pattern: #[deny(warnings)]"]
related_syntax: ["#[expect(...)]", "#[meta] / #![meta]"]
see_also: ["Anti-pattern: #[deny(warnings)]", "#[expect(...)]"]
---

## Explanation

`#[allow(...)]`, `#[warn(...)]`, `#[deny(...)]`, and `#[forbid(...)]` are
one family: four **levels** of increasing severity, each taking one or
more lint names and applying that level to those lints for the item the
attribute decorates (and everything nested inside it, unless a narrower
inner scope overrides the level for itself). Every specific named lint —
`dead_code`, `unused_variables`, `missing_docs`, a Clippy lint like
`clippy::needless_clone`, and hundreds more — can be assigned any of these
four levels:

- **`allow`** — silences the lint entirely for the annotated scope, even
  if it would otherwise fire. `#[allow(dead_code)]` on a function that
  genuinely isn't called yet (a stub for work in progress, say) suppresses
  the warning without deleting the function.
- **`warn`** — the level most lints already have by default: a warning is
  printed, but compilation still succeeds. Explicitly writing `#[warn(...)]`
  is mostly useful to *raise* a lint that defaults to `allow` up to
  visible, or to restate `warn` inside a scope that an enclosing `deny`
  would otherwise apply to.
- **`deny`** — turns the named lint into a hard compile error for the
  annotated scope: code that would merely warn now fails to build at all.
- **`forbid`** — everything `deny` does, **plus** it prevents any nested
  scope from lowering the lint back down to `allow` or `warn` — an inner
  `#[allow(some_lint)]` inside a `#[forbid(some_lint)]` scope is itself a
  compile error, rather than silently taking effect. This is the strongest
  level: a `forbid`den lint cannot be locally reopened anywhere beneath it.

All four apply the same way whether written as an outer attribute on one
item (`#[deny(unused_must_use)]` above a single function) or an inner
attribute at a crate root (`#![deny(unused_must_use)]`, applying
crate-wide) — see [`#[meta] / #![meta]`](attribute-syntax.md) for that
general outer/inner distinction.

**The command line does the same job, without editing source.**
`rustc`/`cargo build`/`cargo clippy` accept `-A`, `-W`, `-D`, `-F` flags
(or the `RUSTFLAGS` environment variable, e.g.
`RUSTFLAGS="-D warnings" cargo build`) that set a lint level for an entire
build without touching any attribute in the source. This is the right
tool for a CI-only, job-specific gate ("fail this build if there are any
warnings") that shouldn't be baked into the source every downstream
consumer also compiles with. This page covers the **attribute** form in
depth, since that's the form that travels with the source and applies
consistently regardless of how the crate is invoked.

**The specific danger of denying the whole `warnings` group** — writing
`#![deny(warnings)]` instead of naming specific lints — is covered in
depth on the
[Anti-pattern: #[deny(warnings)]](../../concepts/design-patterns-idioms/anti-pattern-deny-warnings.md)
concept page: it denies every lint a *future* compiler or Clippy release
might add, not just the ones known today, which can turn a routine
toolchain upgrade into a broken build. That page makes the full argument;
this page is about the four-level mechanism itself.

## Usage examples

### Allowing and denying specific lints

```
#[allow(dead_code)] // <- silences the unused-code warning for this one function
fn planned_for_next_release() {}

#[deny(unused_must_use)] // <- turns this specific lint into a hard error, just for this function
fn must_check_the_result() -> Result<(), &'static str> {
    Ok(())
}
```

### Designing a public API

A crate wants a small, curated set of lints to be hard errors — signaling
"these are not negotiable" — without denying the open-ended `warnings`
group and risking a future toolchain upgrade breaking the build on its
own.

```
// AVOID: denies every lint any future compiler/Clippy release might add
// #![deny(warnings)]

// PREFER: names exactly the lints this crate treats as non-negotiable
#![deny(unused_must_use, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)] // <- tracked, but not yet a hard error

/// Parses a port number from a configuration string.
pub fn parse_port(input: &str) -> Result<u16, std::num::ParseIntError> {
    input.trim().parse()
}
```

Naming specific lints means a compiler or Clippy upgrade
that introduces a brand-new warn-by-default lint doesn't retroactively
turn previously-clean code into a build failure — see
[Anti-pattern: #[deny(warnings)]](../../concepts/design-patterns-idioms/anti-pattern-deny-warnings.md)
for the full argument against the blanket `warnings` group this contrasts
with.

### Implementing traits

A crate exposes an `unsafe` FFI wrapper module where a specific safety
lint must never be silenced by accident, even by a contributor working
several call-sites deep inside that module — `forbid` is what makes an
inner `#[allow]` on that lint itself a compile error, rather than quietly
succeeding.

```
#[forbid(unsafe_op_in_unsafe_fn)] // <- stronger than deny: can't be reopened by any nested scope
mod ffi {
    pub unsafe fn read_register(address: usize) -> u32 {
        // AVOID: this inner #[allow] would itself fail to compile under #[forbid] above
        // #[allow(unsafe_op_in_unsafe_fn)]
        unsafe { std::ptr::read_volatile(address as *const u32) }
    }
}
```

`deny` alone still allows a nested scope to locally
`allow` the same lint back, which is appropriate for most lints but wrong
for a safety-critical one a team wants to guarantee is enforced
everywhere beneath a module boundary; the
[rustc book's lint levels chapter](https://doc.rust-lang.org/rustc/lints/levels.html)
documents `forbid` as specifically closing that reopening loophole.

## Embedded Rust Notes

**Full support.** All four levels are pure compile-time lint configuration
with zero runtime effect, so they apply identically in `#![no_std]` code.
`#[allow(dead_code)]` shows up especially often in embedded crates around
hardware register definitions and HAL boilerplate generated for a whole
chip family, where plenty of fields/functions are legitimately unused by
any single application.

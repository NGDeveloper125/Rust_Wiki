---
title: "#[recursion_limit = \"N\"] / #[type_length_limit = \"N\"]"
kind: attribute
embedded_support: full
groups: ["Macros & Metaprogramming"]
related_concepts: []
related_syntax: ["macro_rules!"]
see_also: []
---

## Explanation

Both attributes are inner attributes placed at a crate root
(`#![recursion_limit = "N"]`, `#![type_length_limit = "N"]`) that raise a
numeric limit the compiler otherwise enforces to keep certain kinds of
unbounded recursion from spinning forever or blowing the stack during
compilation. Neither has any effect on the compiled program's runtime
behavior — both are purely compile-time ceilings.

**`#![recursion_limit = "N"]`** raises the limit on how deeply the
compiler will recurse while expanding a recursive `macro_rules!` macro, or
while resolving a deeply nested chain of trait bound implications. The
default (128 at the time of writing, though the exact number can change
between compiler versions) is generous enough for the overwhelming
majority of code; hitting it produces an explicit compiler error naming
`recursion_limit` and suggesting a specific new value. This tends to
surface with `macro_rules!` macros implemented recursively over a
variadic-like list of tokens (processing one item per recursive
expansion) once that list gets long enough, or occasionally with very
deep chains of generic trait bounds implying other trait bounds.

**`#![type_length_limit = "N"]`** raised a limit on how large the compiler
would let a single monomorphized type's internal name grow — historically
triggered by, for instance, deeply nested `impl Trait` return types or
long iterator-adapter chains whose monomorphized type name grows with
every `.map()`/`.filter()` link in the chain. This is largely a historical
concern on modern compilers, which have grown considerably more tolerant
of long type names on their own; it's now rarely the actual fix a compiler
error suggests, though it can still appear in older code or with unusually
extreme cases of type-name growth.

**Be honest about both:** these are escape hatches reached for only
*after* hitting a specific compiler error that names the limit and
suggests raising it — not something to add preemptively "just in case."
Raising either limit doesn't fix a design problem; it just buys more room
for something (often a macro that could be restructured iteratively
instead of recursively) to keep going.

## Basic usage example

```
#![recursion_limit = "256"] // <- raises the macro-expansion/trait-resolution recursion ceiling

macro_rules! count_tokens {
    () => { 0 };
    ($_head:tt $($tail:tt)*) => { 1 + count_tokens!($($tail)*) };
}

const N: usize = count_tokens!(a b c d e f g h); // one recursive expansion per token
```

## Best practices & deeper information

### Scenario: Designing a public API

A `macro_rules!` macro that recursively counts or processes a long
variadic-style list of items hits the default recursion limit once
enough items are passed at a call site — raising `recursion_limit` is the
direct fix, reached for only after the compiler's own error names it.

```
#![recursion_limit = "512"] // <- raised only after hitting the default limit's compiler error

macro_rules! sum_of {
    () => { 0 };
    ($head:expr $(, $tail:expr)*) => { $head + sum_of!($($tail),*) };
}

fn total() -> i32 {
    // a long, generated call site with many arguments could exceed the default recursion depth
    sum_of!(1, 2, 3, 4, 5, 6, 7, 8)
}
```

**Why this way:** the compiler's own error message for exceeding the
default recursion limit names `recursion_limit` explicitly and suggests a
concrete replacement value, which the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/limits.html#the-recursion_limit-attribute)
documents as the intended way to respond — raising the crate-wide limit is
appropriate once a specific macro's legitimate recursion depth is known
to exceed the default, rather than guessing at a number in advance.

## Embedded Rust Notes

**Full support.** Both attributes are pure compile-time limits with zero
runtime footprint, so they apply identically whether or not the crate
links `std`. Embedded crates that lean heavily on recursive
`macro_rules!` macros for generating repetitive peripheral/register
bindings across a whole chip family are, in practice, one of the more
common places `#![recursion_limit]` actually needs raising.

---
title: "for<'a> Type"
kind: lifetime
embedded_support: full
groups: ["Types & Data Structures", "Traits & Polymorphism"]
related_concepts: [Generics, "Trait bounds", Closures]
related_syntax: [where, "+", trait]
see_also: ["Trait bounds"]
---

## Explanation

`for<'a> Type` is a **higher-ranked trait bound** (HRTB): a bound that
says "this must hold for *every* lifetime `'a`, not one specific lifetime
chosen ahead of time." It shows up almost exclusively on trait bounds
involving a reference-taking closure or function, most commonly `Fn`/
`FnMut`/`FnOnce` — a bound like `F: for<'a> Fn(&'a str) -> bool` requires
`F` to work for any lifetime the caller ends up picking, not one fixed
lifetime chosen where the bound is written, as the complete function
below demonstrates.

To see why this is needed, compare it to the bound without `for<'a>`:
`F: Fn(&'a str) -> bool` would require `'a` to be some *one* concrete
lifetime, fixed at the point the bound is written — but at that point,
no such lifetime exists yet, because the string reference `f` will
actually be called with doesn't exist until `apply` runs and constructs
one from data with its own, unrelated lifetime. A single fixed `'a`
can't describe "whatever lifetime a locally-created `&str` happens to
have inside this function body" — that lifetime is different on every
call, and often shorter than anything nameable in the surrounding
function signature. `for<'a>` sidesteps the whole problem: instead of
picking one lifetime up front, it requires `F` to work no matter which
lifetime shows up, which is exactly the guarantee needed to call `f` with
a reference that's only born inside `apply`'s own body.

In practice, the compiler infers `for<'a>` automatically for the common
`Fn(&str) -> bool`-shaped bound — writing `F: Fn(&str) -> bool` (lifetime
elided) implicitly means `F: for<'a> Fn(&'a str) -> bool`. The explicit
`for<'a>` syntax becomes necessary to write out by hand once the bound
needs to name that lifetime elsewhere in the same signature, or once
elision genuinely can't produce the higher-ranked bound on its own (for
instance, a bound spanning multiple arguments that must share one
universally-quantified lifetime).

## Basic usage example

```
fn find_first<'s>(text: &'s str, matches: impl for<'a> Fn(&'a str) -> bool) -> Option<&'s str> {
    // <- `for<'a>`: the closure must accept a &str of ANY lifetime, not one fixed lifetime
    text.split_whitespace().find(|word| matches(word))
}

find_first("sensor offline retry", |w| w.starts_with("r"));
```

## Best practices & deeper information

### Scenario: Writing generic code

A function that accepts a validation closure and calls it with several
short-lived string slices, each borrowed only for the duration of one
iteration, needs the closure's bound to hold for all of those
independently-scoped borrows at once — exactly what `for<'a>` expresses.

```
fn validate_all<F>(entries: &[String], is_valid: F) -> bool
where
    F: for<'a> Fn(&'a str) -> bool, // <- must accept a &str with whatever lifetime each iteration produces
{
    entries.iter().all(|entry| is_valid(entry))
}

let readings = vec!["21.5".to_string(), "22.0".to_string(), "19.8".to_string()];
let all_positive = validate_all(&readings, |value| {
    value.parse::<f64>().map(|v| v > 0.0).unwrap_or(false)
});
```

**Why this way:** each call to `is_valid` inside the loop passes a
reference whose lifetime is tied to that specific iteration — no single
named lifetime in `validate_all`'s own signature could stand in for all
of them, so the bound has to be universally quantified over every
possible lifetime via `for<'a>`, as the
[Rust Reference's higher-ranked trait bounds section](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds)
describes.

### Scenario: Designing a public API

A callback-taking API that stores the closure and invokes it later, with
borrowed data whose lifetime differs on every invocation, must bound the
closure with `for<'a>` rather than a single named lifetime parameter on
the containing function.

```
struct Parser<F> {
    on_token: F,
}

impl<F> Parser<F>
where
    F: for<'a> Fn(&'a str), // <- one bound covering every future call, each with its own lifetime
{
    fn run(&self, input: &str) {
        for token in input.split_whitespace() {
            (self.on_token)(token); // <- `token`'s lifetime is different (and shorter) on every iteration
        }
    }
}

let parser = Parser { on_token: |t: &str| println!("token: {t}") };
parser.run("start measuring stop");
```

**Why this way:** giving `Parser` its own named lifetime parameter and
bounding `F: Fn(&'p str)` against it would tie every call to one fixed
lifetime `'p` chosen when `Parser` is constructed — but the `&str`s
passed to `on_token` are created fresh inside `run`, with a shorter
lifetime than `'p` could ever be; `for<'a>` is the only bound shape that
accepts a closure usable across all of those independently-scoped calls.

## Embedded Rust Notes

**Full support.** HRTBs are resolved entirely at compile time with no
runtime representation — no `std`/allocator dependency. They appear in
`no_std` code any time a generic function takes a closure over borrowed
data with a per-call lifetime, exactly as on a hosted target.

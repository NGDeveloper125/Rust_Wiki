---
title: "cfg!"
kind: macro
embedded_support: full
groups: ["Compile-time Introspection", "Macros & Metaprogramming"]
related_concepts: []
related_syntax: ["#[cfg(...)]"]
see_also: ["#[cfg(...)]"]
---

## Explanation

`cfg!(predicate)` evaluates a configuration predicate — the same
predicate grammar used inside `#[cfg(...)]`, like `target_os = "linux"`,
`target_arch = "arm"`, `feature = "async"`, combined with `all(...)`,
`any(...)`, `not(...)` — at compile time, producing a plain `bool` that
can be used anywhere an expression is expected: as an `if` condition, a
match guard, or stored in a variable.

That's the whole difference from the attribute, and it's worth stating
precisely: [`#[cfg(...)]`](../attributes/cfg-attribute.md) is applied to
an *item* (a function, a module, a struct field, a whole block of code)
and, when its predicate doesn't hold, deletes that item from the source
entirely before the rest of compilation ever sees it — code inside a
non-matching `#[cfg(...)]` block doesn't need to type-check, doesn't need
to compile, and can even reference types or functions that don't exist on
the current target. `cfg!(...)`, in contrast, is an ordinary expression:
the compiler evaluates the predicate to `true` or `false`, but every
branch of the surrounding code — the `if` block and its `else` — is still
fully present, fully type-checked, and fully compiled; only which branch
*runs* is decided by the boolean's value, at runtime, exactly like any
other `if`.

That "both branches must compile" property is the main practical
consequence: reaching for `cfg!(...)` where a symbol genuinely doesn't
exist on some target (an OS-specific function, a target-only crate)
doesn't work, because the untaken branch still has to reference
something that compiles — `#[cfg(...)]` on the whole function or module
is the right tool whenever a branch shouldn't even attempt to compile on
some target.

## Usage examples

### Choosing a path separator at compile time

```
let path_separator = if cfg!(target_os = "windows") { '\\' } else { '/' };
// <- both branches are compiled on every target; cfg! only decides, at compile time, which one *runs*
```

### Designing a public API

A logging helper picks a platform-appropriate line ending at compile
time, without needing two separately-compiled versions of the function
gated by `#[cfg]`.

```
fn format_log_line(message: &str) -> String {
    let newline = if cfg!(target_os = "windows") { "\r\n" } else { "\n" };
    format!("{message}{newline}") // <- both branches type-check on every target; cfg! just picks one
}

let line = format_log_line("service started");
```

The
[std docs](https://doc.rust-lang.org/std/macro.cfg.html) recommend
`cfg!` for exactly this kind of small, always-compilable behavioral
branch — since both platforms' logic is trivial and compiles fine
everywhere, a runtime `cfg!` check inside one shared function is simpler
than maintaining two `#[cfg]`-gated copies of the whole function.

### Handling and propagating errors

An internal helper reports a more detailed, less stable error message in
debug builds and a terse, stable one in release, branching on
`cfg!(debug_assertions)` rather than a separate build-time flag.

```
fn describe_failure(code: i32) -> String {
    if cfg!(debug_assertions) {
        format!("operation failed with raw code {code} (debug build)") // <- verbose form; still compiled into release, just unused there
    } else {
        format!("operation failed (code {code})")
    }
}
```

The
[std docs](https://doc.rust-lang.org/std/macro.cfg.html) point to
`cfg!(debug_assertions)` as the standard way to vary *behavior* by build
profile without a custom Cargo feature — both message forms live in the
same compiled binary per profile, and which one runs is chosen at
runtime by the flag baked in at compile time.

## Embedded Rust Notes

**Full support.** `cfg!` is resolved entirely at compile time against the
target's configuration — it has no runtime dependency on `std` or an OS,
and is used constantly in embedded code to pick behavior (e.g.
`cfg!(target_arch = "arm")`) inside a function shared across targets.

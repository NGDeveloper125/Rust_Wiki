---
title: "A tour of the ? operator: error handling that gets out of your way"
author: "Nimrod Gaudin"
github: "NGDeveloper125"
date: "2026-07-25"
summary: "How Rust's ? operator turns verbose error plumbing into a single character — what it desugars to, how it works for both Result and Option, and the From conversion that lets it bridge error types."
tags: ["error-handling", "result", "?", "beginner"]
---

Every language has to answer the same awkward question: *what happens when a
function can't do what you asked?* Rust's answer is unusually honest — errors
are ordinary values, carried in [`Result<T, E>`](../concepts/error-handling/result.md)
and [`Option<T>`](../concepts/error-handling/option.md), and you handle them
with the same tools you use for everything else. The catch is that honesty is
verbose. Threading a possible failure up through five call frames by hand is
miserable.

The [`?` operator](../syntax/operators/question-mark.md) exists to make that
misery disappear without hiding the error. This article is a tour of what it
actually does.

## The problem it solves

Here's error handling written out the long way — read a number from a string,
double it, and hand back either the value or whatever went wrong:

```
fn double(input: &str) -> Result<i64, std::num::ParseIntError> {
    let n = match input.trim().parse::<i64>() {
        Ok(n) => n,          // <- pull the value out on success
        Err(e) => return e.into(), // <- ...or bail early on failure
    };
    Ok(n * 2)
}
```

That `match` is pure ceremony. It says nothing about *doubling* — it's all
plumbing. Now the same function with `?`:

```
fn double(input: &str) -> Result<i64, std::num::ParseIntError> {
    let n = input.trim().parse::<i64>()?; // <- unwrap-or-return, in one char
    Ok(n * 2)
}
```

The `?` reads as *"give me the `Ok` value, or return the `Err` from this
function right now."* The happy path stays front and center; the failure path
shrinks to a single glyph.

## What ? actually desugars to

`?` is not magic — it's a well-defined rewrite. For a `Result`, `expr?`
expands to roughly:

```
match expr {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)), // <- note the conversion!
}
```

Two things are worth pausing on:

- It **returns from the enclosing function**, not just the current expression.
  `?` only makes sense inside a function that itself returns a `Result` (or
  `Option`, or another type implementing `Try`).
- The error is passed through `From::from` before being returned. That single
  [conversion](../concepts/error-handling/custom-error-types.md) is what makes
  `?` compose across *different* error types — more on that below.

## It works for Option too

The same operator threads `None` through a function that returns an
[`Option<T>`](../concepts/error-handling/option.md):

```
fn first_char_upper(s: &str) -> Option<char> {
    let c = s.chars().next()?; // <- None here returns None from the function
    Some(c.to_ascii_uppercase())
}
```

Here `expr?` unwraps `Some(v)` to `v`, and on `None` returns `None` from the
function. You can't mix them implicitly — a `?` on an `Option` in a function
that returns `Result` won't compile — which keeps the "what can this function
fail with?" contract visible in the signature.

## The From conversion is the whole trick

Real programs fail in more than one way. A function that reads a file and
parses it can hit an [I/O error](../concepts/error-handling/the-error-trait.md)
*or* a parse error. Because `?` runs every error through `From::from`, you can
collect several underlying error types into one:

```
#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self { ConfigError::Io(e) }
}
impl From<std::num::ParseIntError> for ConfigError {
    fn from(e: std::num::ParseIntError) -> Self { ConfigError::Parse(e) }
}

fn load_port(path: &str) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?; // <- io::Error -> ConfigError
    let port = text.trim().parse::<u16>()?;     // <- ParseIntError -> ConfigError
    Ok(port)
}
```

Both `?` sites return the *same* `ConfigError`, each converting from its own
source error automatically. This is the standard shape of a
[custom error type](../concepts/error-handling/custom-error-types.md), and
it's why implementing `From` for your error enum pays for itself immediately.

## When *not* to reach for it

`?` propagates; it does not *handle*. If the caller is the right place to make
a decision, don't push the error further up just because `?` is convenient:

- At the top of the program, you eventually have to deal with the value. `main`
  itself can return `Result<(), E>`, so `?` even works there — a returned
  `Err` is printed and sets a non-zero exit code.
- When a failure is genuinely unrecoverable (a violated invariant, not bad
  input), a [panic](../concepts/error-handling/panic-and-unwinding.md) is the
  honest choice, not a `Result` you never inspect.

## The whole thing in one line

`?` is one character standing in for a `match`, an early `return`, and a `From`
conversion. Learn to read it as those three things and Rust's error handling
stops feeling verbose and starts feeling like what it is: ordinary values,
moved around with very little ceremony.

---
title: "anyhow"
version: "1.0"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-07-29"
summary: "One error type for applications: `anyhow::Error` holds any error, keeps the whole chain of causes, and lets `?` work everywhere without writing a single `From` impl."
categories: ["error-handling", "application", "beginner"]
repository: "https://github.com/dtolnay/anyhow"
---

## Overview

`anyhow` answers a question every Rust application hits early: *what type do I
put in the error slot of [`Result<T, E>`](../concepts/error-handling/result.md)?*
Once a function can fail in two different ways — an IO error here, a parse error
there — the [`?` operator](../syntax/operators/question-mark.md) stops compiling
until you define an error enum and a `From` impl for every source error. That is
the right amount of work for a library. For an application, it's ceremony that
buys nothing: nobody matches on your error variants, they print them.

`anyhow::Error` is the alternative: a single, non-generic error type that can
hold *any* `Send + Sync + 'static` error implementing
[the `Error` trait](../concepts/error-handling/the-error-trait.md). Because
`From` is implemented for all of them, `?` converts into it automatically, so
one return type covers a whole program. It is
a smart-pointer-sized value (one word wide, the error boxed behind it), it keeps
the full chain of underlying causes rather than flattening them into a string,
and it can capture a backtrace at the point the error was created.

The trade-off is deliberate and worth stating plainly: `anyhow::Error` is
type-erased, so callers can't `match` on what went wrong without downcasting.
That's why the ecosystem's rule of thumb is **`anyhow` in applications and
binaries, [typed errors](../concepts/error-handling/custom-error-types.md) in
libraries** — a library's errors are part of its public API, and its users
deserve something they can match on. The same author's `thiserror` crate is the
usual partner for that side of the line, and the two compose: a library returns
its own enum, and the application that consumes it swallows all of them into
`anyhow::Error`.

Practically it is one dependency with no required transitive dependencies, it
compiles fast, it is one of the most widely used crates in the ecosystem, and it
works in `no_std` builds with `default-features = false` (it still needs
`alloc`).

## When to use it

`anyhow` is the default choice whenever an error's job is to be *reported*
rather than *handled*. Three situations where that's the case:

### Use case: A binary where errors just need to reach the user

`main` can return `Result`, so an application can propagate failures all the way
out and let the runtime print them — no error type of your own, no `From` impls,
`?` across three unrelated error kinds in one function.

```
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    let text = fs::read_to_string("port.txt")?; // <- std::io::Error
    let port: u16 = text.trim().parse()?;       // <- std::num::ParseIntError
    println!("listening on {port}");
    Ok(())
}
```

**Why it fits:** two unrelated error types flow through one `?` each and land in
the same return type. Returning `Result` from `main` prints the error with its
`Debug` formatting on exit, which for `anyhow::Error` means the message, the
chain of causes, and a backtrace if one was captured.

### Use case: Explaining a failure while it bubbles up

A raw `No such file or directory (os error 2)` tells the user nothing about
*which* file or *why* the program wanted it. `anyhow`'s `Context` trait attaches
that explanation at each layer without discarding the original error.

```
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn load_config(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("reading config from {}", path.display()))
}

fn start() -> Result<()> {
    let _cfg = load_config(Path::new("app.toml")).context("starting the server")?;
    Ok(())
}
```

**Why it fits:** the printed error becomes a stack of explanations — *starting
the server*, caused by *reading config from app.toml*, caused by *No such file
or directory* — each layer adding what only it knew. Nothing is lost: the
original `io::Error` is still in the chain and can be downcast back out.

### Use case: Handling one specific failure, reporting the rest

Type erasure doesn't mean giving up control. When exactly one failure deserves
special treatment, downcast for that one and let everything else propagate.

```
use anyhow::{Context, Result};
use std::fs;
use std::io;

fn read_or_default(path: &str) -> Result<String> {
    match fs::read_to_string(path).with_context(|| format!("reading {path}")) {
        Ok(text) => Ok(text),
        Err(e) => match e.downcast_ref::<io::Error>() {
            // <- a missing file is fine; anything else is a real failure
            Some(io) if io.kind() == io::ErrorKind::NotFound => Ok(String::new()),
            _ => Err(e),
        },
    }
}
```

**Why it fits:** you pay the cost of a downcast only at the one place that
actually branches on the cause, instead of maintaining an error enum across the
whole program for the sake of it.

## API map

The whole crate is a handful of items: one error type, one trait, three macros,
and a `Result` alias. Everything below is in the crate root.

### Creating an error

Four ways to make an `anyhow::Error`, from most to least common.

#### `anyhow!`

Builds an `Error` from a format string, or wraps any value that is
`Debug + Display + Send + Sync + 'static`.

```
use anyhow::anyhow;

let name = "config.toml";
let err = anyhow!("{name} is missing a [server] section");
```

**When to use it:** when the failure originates in your own code and there is no
underlying error to wrap — a validation failure, an impossible state. If you're
about to `return Err(anyhow!(...))`, use `bail!` instead.

#### `bail!`

`return Err(anyhow!(...))` in one word. Takes exactly the same arguments as
`anyhow!`.

```
use anyhow::{bail, Result};

fn set_workers(n: usize) -> Result<()> {
    if n == 0 {
        bail!("worker count must be at least 1");
    }
    Ok(())
}
```

**When to use it:** for an early return on a failed check. It's the error-path
equivalent of an early `return`, and it reads better than the `return Err(...)`
it expands to.

#### `ensure!`

Asserts a condition and bails with the given message if it doesn't hold. With no
message, it reports the failed condition verbatim.

```
use anyhow::{ensure, Result};

fn set_workers(n: usize) -> Result<()> {
    ensure!(n >= 1, "worker count must be at least 1, got {n}");
    ensure!(n <= 512); // <- message defaults to "Condition failed: `n <= 512`"
    Ok(())
}
```

**When to use it:** for precondition checks at the top of a function. It is to
`bail!` what `assert!` is to `panic!` — with the crucial difference that it
returns an error instead of unwinding, so it's safe on input you don't control.

#### `Error::msg`

Builds an `Error` from a single `Display + Debug + Send + Sync + 'static` value,
without the macro's format-string handling.

```
use anyhow::Error;

let err = Error::msg("shutting down");
```

**When to use it:** when you already have the message as a value — including in
a non-macro position like `.ok_or_else(Error::msg)` or `.map_err(Error::msg)`.
For a literal or a format string, `anyhow!` reads better.

#### `Error::new`

Wraps an existing error that implements `std::error::Error + Send + Sync +
'static`, preserving it as the source.

```
use anyhow::Error;
use std::io;

let err = Error::new(io::Error::new(io::ErrorKind::Other, "device offline"));
```

**When to use it:** rarely and explicitly — `?` and `.into()` already do this
conversion for you. Reach for it when there's no `?` in play, such as building
an `Error` to store or send somewhere.

### Adding context

The `Context` trait is the reason most people install this crate. It's
implemented for `Result<T, E>` and for [`Option<T>`](../concepts/error-handling/option.md),
and importing it brings both methods into scope.

#### `Context::context`

Wraps the error in an extra layer of explanation, keeping the original as the
cause. On an `Option`, it turns `None` into an error.

```
use anyhow::{Context, Result};
use std::fs;

fn read_key() -> Result<String> {
    let raw = fs::read_to_string("key.pem").context("reading the TLS key")?;
    let first = raw.lines().next().context("the key file is empty")?; // <- on Option
    Ok(first.to_owned())
}
```

**When to use it:** on essentially every `?` that crosses a boundary the caller
can't see through — file paths, URLs, record IDs. Use it when the context value
is cheap to build (a literal, an existing `String`).

#### `Context::with_context`

The same thing, but the context is built by a closure that only runs on the
error path.

```
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}
```

**When to use it:** whenever building the message costs anything — any
`format!`, any allocation. On a hot success path, `context(format!(...))` would
format a message on every call and throw it away; this doesn't.

#### `Error::context`

The same layering, but applied to an `anyhow::Error` you already hold rather
than to a `Result`.

```
use anyhow::{anyhow, Error};

let err: Error = anyhow!("connection reset").context("syncing with upstream");
```

**When to use it:** when you're manipulating an `Error` value directly — in a
`match` arm, or when re-wrapping an error you caught and inspected.

### Result and its helpers

#### `anyhow::Result<T>`

A type alias for `std::result::Result<T, anyhow::Error>`. The second parameter
still defaults, so `Result<T, E>` also works if you need a different error type
in one signature.

```
use anyhow::Result;

fn parse_port(text: &str) -> Result<u16> {
    Ok(text.trim().parse()?)
}
```

**When to use it:** as the return type of essentially every fallible function in
an application. Import it once per module and never spell the error type again.

#### `anyhow::Ok`

A function that constructs an `Ok` already pinned to `anyhow::Error`, so type
inference doesn't need help.

```
use anyhow::Result;

fn parse_all(lines: &[&str]) -> Result<Vec<u16>> {
    lines
        .iter()
        .map(|l| anyhow::Ok(l.trim().parse::<u16>()?)) // <- fixes the closure's error type
        .collect()
}
```

**When to use it:** inside a closure or block whose error type the compiler
can't infer — the classic "type annotations needed" error on an iterator chain
that uses `?`. Everywhere else, plain `Ok` is fine.

### Inspecting an error

#### `Error::chain`

Returns an iterator over this error and every error underneath it, outermost
first.

```
use anyhow::Error;

fn log_causes(err: &Error) {
    for cause in err.chain() {
        eprintln!("- {cause}");
    }
}
```

**When to use it:** when you're formatting an error yourself — a log line, a
JSON field, an HTTP response body — and want each layer separately instead of
the crate's default rendering.

#### `Error::root_cause`

The last error in the chain: the original failure everything else wrapped.

```
use anyhow::Error;

fn is_fatal(err: &Error) -> bool {
    err.root_cause().to_string().contains("disk full")
}
```

**When to use it:** when only the bottom of the chain matters — deciding whether
to retry, or reporting the underlying failure without your own context layers.
`chain().last()` is the same value.

#### `Error::downcast_ref`

Borrows the error back as a concrete type, if that type is anywhere in the
chain.

```
use anyhow::Error;
use std::io;

fn is_not_found(err: &Error) -> bool {
    matches!(err.downcast_ref::<io::Error>(), Some(e) if e.kind() == io::ErrorKind::NotFound)
}
```

**When to use it:** the common way to recover from one specific failure while
letting the rest propagate. `downcast_mut` is the same thing for a mutable
borrow.

#### `Error::downcast`

Takes the concrete error out by value, returning the original `Error` unchanged
in the `Err` case so nothing is lost when the type doesn't match.

```
use anyhow::{Error, Result};
use std::io;

fn recover(err: Error) -> Result<io::Error> {
    err.downcast::<io::Error>() // <- Err(e) hands the original error back
}
```

**When to use it:** when you need to *own* the concrete error — to return it, to
store it, or to call a method that consumes it. Prefer `downcast_ref` when a
borrow is enough.

#### `Error::is`

Whether the chain contains an error of the given type.

```
use anyhow::Error;
use std::io;

fn touched_the_filesystem(err: &Error) -> bool {
    err.is::<io::Error>()
}
```

**When to use it:** for a plain yes/no check where you don't need the error
itself — classifying a failure for a metric or a log level.

#### `Error::backtrace`

The backtrace captured when the error was created, if backtraces were enabled at
runtime.

```
use anyhow::Error;

fn dump(err: &Error) {
    eprintln!("{}", err.backtrace());
}
```

**When to use it:** when you're building your own crash report and want the
capture site. Note that this is opt-in *at run time*: set `RUST_BACKTRACE=1` (or
`RUST_LIB_BACKTRACE=1` to enable it only for errors) or there is nothing to
print. On Rust 1.65 and later it works on stable with no extra Cargo feature.

### Printing an error

#### `{}` — the message only

`Display` prints just the outermost error, with no causes.

```
use anyhow::Error;

fn line(err: &Error) -> String {
    format!("{err}")
}
```

**When to use it:** in a message for the end user, where the chain would be
noise — a CLI's final "error: ..." line, a toast, an HTTP status message.

#### `{:#}` — the whole chain on one line

The alternate `Display` form joins every layer with `: `, outermost first.

```
use anyhow::Error;

fn line(err: &Error) -> String {
    format!("{err:#}") // <- "starting the server: reading app.toml: No such file..."
}
```

**When to use it:** for structured logging, where the full explanation has to
fit in a single field but you still want every cause.

#### `{:?}` — the full report

`Debug` prints the error, a `Caused by:` list, and the backtrace if one was
captured. This is what `main` returning `Result` uses on exit.

```
use anyhow::Error;

fn report(err: &Error) {
    eprintln!("{err:?}");
}
```

**When to use it:** as the last thing a program prints before it dies, and
anywhere you'd want a developer-facing dump. It's the most informative
rendering and the reason `fn main() -> Result<()>` is worth using.

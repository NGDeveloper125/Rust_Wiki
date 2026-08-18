---
title: "thiserror"
version: "2.0.20"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-18"
summary: "A derive for `std::error::Error`. Write the enum, put the message on each variant, and get `Display`, `source()` and the `From` conversions that make `?` work — without the boilerplate, and without the macro appearing in your public API."
categories: ["error-handling", "library", "macros"]
repository: "https://github.com/dtolnay/thiserror"
---

## Overview

A library's errors should be a type its callers can match on: a `ConfigError`
with a `Missing` variant and an `Io` variant, not a string. Writing that by hand
means an `impl Display`, an `impl Error` with `source()`, and a `From` impl for
every error you wrap — mechanical code that has to be kept in step as variants
come and go.

`thiserror` generates it. You declare the enum and put the message on each
variant; the derive writes the impls.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found at {path}")]
    Missing { path: String },
    #[error("could not read the config")]
    Io(#[from] std::io::Error), // <- also generates From<io::Error>
}

let err = ConfigError::Missing { path: "/etc/app.toml".into() };
assert_eq!(err.to_string(), "config file not found at /etc/app.toml");
```

**The property that matters most is that it doesn't appear in your public API.**
The derive produces exactly the impls you would have written by hand, so callers
see `std::error::Error` and nothing else. Adopting it, or dropping it later for
handwritten impls, is not a breaking change — unusual for a derive macro, and
what makes it safe for a library's error type.

It is the library half of a pair with [`anyhow`](anyhow.md), by the same author,
and choosing between them is choosing what the caller does with the error:

- **A library returns `thiserror`.** Callers need to tell "file missing" from
  "permission denied" to decide what to do, and a typed enum is how they do it.
- **An application returns `anyhow`.** Nothing above it matches on the error; it
  gets reported, so one type carrying context is enough.

The two compose rather than compete: a library's `thiserror` enum converts into
an application's `anyhow::Error` for free, because it implements `Error`.

The cost is a proc-macro dependency, so it pulls
[`syn`](syn.md), `quote` and `proc-macro2` into the build graph — though
anything else you derive has already brought them. Nothing of thiserror survives
into your binary. It requires Rust 1.71, and works without `std` by turning off
the default `std` feature, deriving `core::error::Error` instead.

## When to use it

### Use case: A library error callers can match on

The point of a typed error is that the caller can branch. Each variant gets a
message for humans and a shape for code.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("line {line}: expected a number, found `{found}`")]
    NotANumber { line: usize, found: String },
    #[error("line {line}: value {value} is out of range")]
    OutOfRange { line: usize, value: i64 },
}

fn advice(err: &ParseError) -> &'static str {
    match err {
        ParseError::NotANumber { .. } => "fix the syntax",
        ParseError::OutOfRange { .. } => "reduce the value",
    }
}

let err = ParseError::NotANumber { line: 12, found: "abc".into() };
assert_eq!(err.to_string(), "line 12: expected a number, found `abc`");
assert_eq!(advice(&err), "fix the syntax");
```

**Why it fits:** the message and the match arm come from one declaration, so
they cannot drift apart. Adding a variant is one line plus its message, not one
line plus three impl edits.

### Use case: Wrapping lower-level errors so `?` works

Most library errors are caused by another error. `#[from]` generates the
conversion, which is what lets `?` return your type from a function calling into
`std`.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("could not read the file")]
    Io(#[from] std::io::Error),
    #[error("the file is not valid UTF-8")]
    Encoding(#[from] std::str::Utf8Error),
}

fn load(bytes: &[u8]) -> Result<String, LoadError> {
    let text = std::str::from_utf8(bytes)?; // <- Utf8Error converts itself
    Ok(text.to_owned())
}

let err = load(&[0xff, 0xfe]).unwrap_err();
assert_eq!(err.to_string(), "the file is not valid UTF-8");

// The cause is preserved, not flattened into the message.
use std::error::Error;
assert!(err.source().is_some());
```

**Why it fits:** `#[from]` implies `#[source]`, so one attribute buys both the
conversion and the chain. The underlying error stays reachable through `source()`
for anything printing the full cause list, while your message stays readable on
its own.

### Use case: A stable public error over a private representation

A public enum is part of your API: adding a variant breaks anyone matching
exhaustively. Wrapping a private enum in an opaque struct keeps the freedom to
change it.

```
use thiserror::Error;

// Public, opaque, and cheap to keep compatible.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct StoreError(ErrorRepr);

impl StoreError {
    /// Expose only what callers genuinely need to branch on.
    pub fn is_retryable(&self) -> bool {
        matches!(self.0, ErrorRepr::Timeout { .. })
    }
}

// Private: new variants here break nobody.
#[derive(Debug, Error)]
enum ErrorRepr {
    #[error("timed out after {ms}ms")]
    Timeout { ms: u64 },
    #[error("connection closed")]
    Closed,
}

let err = StoreError(ErrorRepr::Timeout { ms: 500 });
assert_eq!(err.to_string(), "timed out after 500ms");
assert!(err.is_retryable());
```

**Why it fits:** `transparent` forwards `Display` and `source` straight through,
so the wrapper costs nothing at the surface, and the accessor exposes the one
distinction callers actually need. `#[non_exhaustive]` on a public enum is the
lighter alternative when you're happy for callers to see the variants at all.

## API map

Everything `thiserror` offers is an attribute — there are no functions to call,
and the derive generates ordinary impls of standard-library traits. The entries
below are those attributes and what each one generates.

### The derive

#### `#[derive(Error)]`

Generates `impl Display` from your `#[error]` messages and `impl
std::error::Error` with `source()` where a source field exists.

```
use thiserror::Error;

#[derive(Debug, Error)]
#[error("the widget is jammed")]
pub struct WidgetError;

// It is a std error, so it composes with everything expecting one.
fn as_boxed() -> Box<dyn std::error::Error> {
    Box::new(WidgetError)
}

assert_eq!(WidgetError.to_string(), "the widget is jammed");
assert_eq!(as_boxed().to_string(), "the widget is jammed");
```

**When to use it:** on any error type a caller might inspect. `Debug` is required
by the `Error` trait, so derive it alongside — that pair is the whole ceremony.
It works on enums, structs with named fields, tuple structs and unit structs
alike.

### Writing the message

#### `#[error("...")]`

The `Display` message: on each variant of an enum, or on the type for a struct.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("disconnected")]
    Disconnected,
    #[error("queue is full")]
    Full,
}

assert_eq!(Error::Disconnected.to_string(), "disconnected");
```

**When to use it:** on every variant. Write it lowercase and without a trailing
full stop — errors get embedded in larger messages (`failed to start: queue is
full`), where a capital letter mid-sentence reads badly. Describe what went
wrong, not what the caller should do about it.

#### Interpolating fields

`{name}` and `{0}` refer to the error's own fields, with the usual format specs.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown key `{0}`")]
    UnknownKey(String),
    #[error("expected {expected:?}, found {found:?}")]
    Mismatch { expected: String, found: String },
    #[error("{0} exceeds the limit of {max}", max = i32::MAX)]
    TooLarge(i64),
}

assert_eq!(Error::UnknownKey("host".into()).to_string(), "unknown key `host`");
assert_eq!(
    Error::Mismatch { expected: "int".into(), found: "str".into() }.to_string(),
    "expected \"int\", found \"str\"",
);
assert_eq!(
    Error::TooLarge(9_000_000_000).to_string(),
    "9000000000 exceeds the limit of 2147483647",
);
```

**When to use it:** whenever the message needs the specific value — the key that
was missing, the limit exceeded. This is a reason to prefer a typed error over a
string: the value stays a field the caller can read, and is only formatted when
someone prints it. Extra named arguments may be arbitrary expressions, as `max`
shows.

#### `#[error(transparent)]`

Forwards both `Display` and `source()` to the single field, adding no message of
its own.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("bad config")]
    Config,
    #[error(transparent)]
    Other(#[from] std::io::Error), // <- prints exactly as the io::Error does
}

let err: Error = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file").into();
assert_eq!(err.to_string(), "no such file");
```

**When to use it:** for an "anything else" variant, and for the opaque-wrapper
pattern. Reach for it only when you have nothing to add — a variant saying
"could not read the config" and naming the file is more useful to a reader than
one passing an `io::Error` through unchanged.

### Sources and conversions

#### `#[from]`

Generates a `From` impl for the field's type, and marks that field as the source.

```
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io failed")]
    Io(#[from] std::io::Error),
}

fn read() -> Result<String, Error> {
    Ok(std::fs::read_to_string("/nonexistent")?) // <- ? converts for us
}

assert!(read().is_err());
```

**When to use it:** when the wrapped error's type identifies the variant on its
own, which is what makes the conversion unambiguous. Only one variant per source
type can carry it — two variants both taking `#[from] io::Error` won't compile,
because `?` could not choose between them. Use `#[source]` for the second.

#### `#[source]`

Marks the field returned by `source()`, without generating a conversion.

```
use thiserror::Error;

#[derive(Debug, Error)]
#[error("could not load {path}")]
pub struct LoadError {
    path: String,
    #[source] // <- a field named `source` is detected without the attribute
    cause: std::io::Error,
}

use std::error::Error as _;
let err = LoadError {
    path: "/etc/app.toml".into(),
    cause: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
};
assert_eq!(err.to_string(), "could not load /etc/app.toml");
assert_eq!(err.source().unwrap().to_string(), "denied");
```

**When to use it:** when the variant carries context beyond the wrapped error —
here the path, which the `io::Error` doesn't know. Preferable to `#[from]`
whenever the conversion would lose that context, because `?` cannot supply it.
Name the field `source` and the attribute itself is optional.

#### The error chain

`source()` links errors into a chain a caller can walk to the root cause.

```
use std::error::Error as _;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("inner failure")]
struct Inner;

#[derive(Debug, Error)]
#[error("outer failure")]
struct Outer(#[from] Inner);

let err = Outer(Inner);
let mut chain = vec![err.to_string()];
let mut cause = err.source();
while let Some(c) = cause {
    chain.push(c.to_string());
    cause = c.source();
}

assert_eq!(chain, ["outer failure", "inner failure"]);
```

**When to use it:** it is generated for you — what matters is that your message
must not repeat the source's. Joining the chain is the caller's job, and
duplicating it yields "io failed: io failed" when something prints the whole
list.

### Shapes

#### Struct and unit errors

The derive is not limited to enums.

```
use thiserror::Error;

#[derive(Debug, Error)]
#[error("the pool is exhausted")]
pub struct Exhausted;

#[derive(Debug, Error)]
#[error("expected {expected} bytes, got {got}")]
pub struct ShortRead { pub expected: usize, pub got: usize }

#[derive(Debug, Error)]
#[error("invalid opcode {0:#04x}")]
pub struct BadOpcode(pub u8);

assert_eq!(Exhausted.to_string(), "the pool is exhausted");
assert_eq!(ShortRead { expected: 8, got: 3 }.to_string(), "expected 8 bytes, got 3");
assert_eq!(BadOpcode(0x1f).to_string(), "invalid opcode 0x1f");
```

**When to use it:** a struct when the operation has exactly one failure mode — it
saves the caller a `match` with one arm. Start there and switch to an enum when a
second mode appears; the change is additive for you and mechanical for callers.

#### Generic errors

Type parameters work, with the bounds you declare carried onto the impls.

```
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("rejected value {value:?}")]
pub struct Rejected<T: Debug> {
    pub value: T,
}

assert_eq!(Rejected { value: 42 }.to_string(), "rejected value 42");
assert_eq!(Rejected { value: "abc" }.to_string(), "rejected value \"abc\"");
```

**When to use it:** for an error carrying the value that failed — a parser
returning the token it could not accept. Keep the bounds honest: the `Error`
trait needs `Debug + Display`, so a generic error is only as usable as its
parameter allows.

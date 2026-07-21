---
title: "Custom error types"
area: "Error Handling"
embedded_support: full
groups: ["Error Handling", "Handling Errors & Failure", "Error Propagation"]
related_syntax: ["?", enum]
see_also: ["The Error trait", "Result<T, E>", "The ? operator (concept angle)", "Enums (algebraic data types)"]
---

## Explanation

Designing your own `E` in [`Result<T, E>`](result.md) means choosing what
a failure actually *is* as a type. `Result<T, String>` compiles and works,
but it forces every caller back to reading and possibly parsing a
message to find out what went wrong. The idiomatic choice for real code
is a purpose-built type — almost always an
[enum](../types-data-modeling/enums-algebraic-data-types.md), one variant
per distinct way the operation can fail — so callers can `match` on what
happened instead of inspecting text.

The payoff is that a matchable error type lets calling code make actual
decisions: retry on a `Timeout` variant, prompt the user again on
`InvalidInput`, escalate on anything else. It can also carry structured
data specific to the failure — which field was invalid, which line
number, what the offending value was — instead of squeezing everything
into a formatted string up front and losing the structure permanently.

Designing an error type follows the same principles as designing any
enum: one variant per genuinely distinct failure mode, no more. What
makes it specifically an *error* type, and lets it interoperate with the
rest of the ecosystem — `?`, `Box<dyn Error>`, error-reporting crates —
is implementing the standard library's
[`Error` trait](the-error-trait.md). This page focuses on shaping the
type's variants and data; see that page for the trait implementation
itself, both by hand and via `thiserror`.

`thiserror` exists to remove the boilerplate of that trait
implementation: instead of hand-writing `Display` and `Error` impls (and
often `From` conversions for each underlying error a variant wraps), its
derive macro generates them from attributes on the enum. It's the
blessed crate for this exact job in library code, per this wiki's crate
policy for the errors scenario.

Beyond picking variants, a public error enum has its own API-design
questions: whether to mark it `#[non_exhaustive]` so new variants can be
added later without a breaking change, and whether to wrap an underlying
error (preserving the original cause) rather than discarding it into a
plain string. Application code that doesn't need a reusable, matchable
error type at all often reaches for `anyhow` instead of defining one —
see the crate policy note on the [`Result`](result.md) and
[`?`](the-question-mark-operator.md) pages for where that line falls.

## Basic usage example

```
// [dependencies] thiserror = "2"
use thiserror::Error;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String), // <- one variant per distinct failure mode
    #[error("invalid value for `{field}`: {reason}")]
    InvalidValue { field: String, reason: String },
}

fn load(path: &str) -> Result<String, ConfigError> {
    if path.is_empty() {
        return Err(ConfigError::NotFound(path.to_string()));
    }
    Ok(format!("contents of {path}"))
}
```

## Best practices & deeper information

### Scenario: Handling and propagating errors

A library function that talks to a config file can fail for several
distinct reasons; `thiserror` derives `Display` and `Error` (and `From`
conversions) so the error type stays declarative instead of hand-written
boilerplate.

```
// [dependencies] thiserror = "2"
use thiserror::Error;
use std::num::ParseIntError;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("could not read config file: {0}")]
    Io(#[from] std::io::Error), // <- #[from] gives ConfigError a `From<io::Error>`, so `?` can convert into it
    #[error("invalid port number: {0}")]
    InvalidPort(#[from] ParseIntError),
}

fn load_port(path: &str) -> Result<u16, ConfigError> {
    let contents = std::fs::read_to_string(path)?; // <- `?` converts io::Error via the derived From impl
    Ok(contents.trim().parse()?) // <- `?` converts ParseIntError via the derived From impl too
}
```

**Why this way:** `thiserror` derives the message wording and the `Error`
impl (plus `From` conversions via `#[from]`) from attributes instead of
hand-written boilerplate, which is exactly the blessed use of the crate
for the errors scenario in library code — see this wiki's crate policy
and the [`thiserror` docs](https://docs.rs/thiserror/latest/thiserror/).

### Scenario: Designing a public API

A public error enum needs room to grow — a future release may add a new
failure variant — so it's marked `#[non_exhaustive]` to keep that from
being a breaking change for existing `match` statements.

```
#[derive(Debug)]
#[non_exhaustive] // <- callers can't exhaustively match without a wildcard arm, so new variants aren't breaking
pub enum UploadError {
    TooLarge { limit_bytes: u64 },
    Rejected(String),
}

fn describe(error: &UploadError) -> String {
    match error {
        UploadError::TooLarge { limit_bytes } => format!("exceeds {limit_bytes} bytes"),
        UploadError::Rejected(reason) => format!("rejected: {reason}"),
        _ => "unknown upload error".to_string(), // <- required because of #[non_exhaustive]
    }
}
```

**Why this way:** without `#[non_exhaustive]`, adding a new variant to a
public error enum is a breaking change for every downstream `match`;
marking it up front trades a little match-site verbosity now for the
freedom to extend the error type later without a major version bump, per
the
[API Guidelines' future-proofing section](https://rust-lang.github.io/api-guidelines/future-proofing.html).

### Scenario: Validating input

Validating a sensor reading against its expected range should reject
specific, distinguishable problems, not just a generic "invalid" — so the
custom error type has one variant per kind of invalid reading.

```
#[derive(Debug, PartialEq)]
enum SensorError {
    OutOfRange { value: f64, min: f64, max: f64 },
    NotANumber,
}

fn validate_reading(raw: &str) -> Result<f64, SensorError> {
    let value: f64 = raw.trim().parse().map_err(|_| SensorError::NotANumber)?;
    if value < -40.0 || value > 125.0 {
        return Err(SensorError::OutOfRange { value, min: -40.0, max: 125.0 });
    }
    Ok(value)
}

assert_eq!(
    validate_reading("200"),
    Err(SensorError::OutOfRange { value: 200.0, min: -40.0, max: 125.0 })
);
```

**Why this way:** a variant per failure kind lets callers branch on what
specifically went wrong (retry parsing, clamp the value, alert on a
sensor fault) instead of pattern-matching a string, which
[Effective Rust](https://effective-rust.com/) recommends over
stringly-typed errors for anything beyond a throwaway script.

## Embedded Rust Notes

**Full support.** A custom error enum is just an enum plus trait impls —
core-language, no allocator required by the type itself. Since Rust 1.81
stabilized `core::error::Error`, even a `#![no_std]` crate can implement
the real `Error` trait directly; using `thiserror` there additionally
requires disabling its default `std` feature, since the crate's `std`
support assumes an allocator and the full `Error` trait location present
on hosted targets.

---
title: "String literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: ["String vs &str"]
related_syntax: [raw-string-literal, escape-sequences]
see_also: [raw-string-literal]
---

## Explanation

A double-quoted string literal produces a `&'static str` — a borrowed
reference to UTF-8 text baked directly into the compiled binary, not an
owned, heap-allocated `String`:

```
let s: &str = "hello, world";
```

To get an owned, growable `String`, convert explicitly: `"hello".to_string()`
or `String::from("hello")`. The literal itself is always `&str`. Escape
sequences (`\n`, `\t`, `\\`, `\"`, `\u{...}`, …) are processed inside a
normal string literal — see [escape sequences](escape-sequences.md); use
a [raw string literal](raw-string-literal.md) when you want backslashes
taken literally.

## Basic usage example

```
let msg = "connected"; // <- string literal: produces `&'static str`, not `String`
```

## Best practices & deeper information

### Scenario: Working with text

A `format!` template is itself a string literal, with captured
identifiers keeping the placeholder and its value next to each other.

```
fn format_status(sensor_id: &str, reading: f64) -> String {
    format!("sensor \"{sensor_id}\" reported {reading:.1}") // <- string literal: the format template itself
}

let msg = format_status("temp-01", 21.53);
assert_eq!(msg, "sensor \"temp-01\" reported 21.5");
```

**Why this way:** captured-identifier syntax (`{sensor_id}`) keeps the
template and its values together instead of separated into positional
arguments, which the
[std `format!` docs](https://doc.rust-lang.org/std/fmt/index.html)
recommend once a template has more than one or two placeholders.

### Scenario: Designing a public API

A function parameter should ask for the least ownership it needs — `&str`
accepts a literal directly, with no allocation, while still working for
an owned `String` through auto-deref.

```
// PREFER: `&str` accepts a literal, a `String`, or a slice, no forced copy
pub fn greet(name: &str) -> String {
    format!("hello, {name}") // <- string literal: the format template
}

// AVOID: `String` forces every caller to allocate, even for a literal like "guest"
pub fn greet_owned(name: String) -> String {
    format!("hello, {name}")
}

let a = greet("guest");                     // <- string literal passed directly, no allocation
let b = greet_owned("guest".to_string());   // caller must allocate just to satisfy the signature
```

**Why this way:** accepting `&str` instead of `String` lets a caller pass
a literal with zero allocation while still accepting an owned `String`
where one already exists — preferring borrowed types in function
signatures is a rule [Effective Rust](https://effective-rust.com/) covers
under its API-design guidance.

### Scenario: Handling and propagating errors

An error message built with a string literal template should include the
offending input, not just describe the category of failure.

```
#[derive(Debug)]
struct ParseError {
    message: String,
}

fn parse_port(input: &str) -> Result<u16, ParseError> {
    input.parse::<u16>().map_err(|_| ParseError {
        message: format!("\"{input}\" is not a valid port number"), // <- string literal: the format template
    })
}
```

**Why this way:** including the offending value in the message gives the
caller enough information to fix the problem without re-deriving it,
which the
[Book's error-handling chapter](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
treats as basic error-message hygiene.

## Embedded Rust Notes

**Full support.** A `&str`/`&'static str` literal needs no allocator at
all — it's baked into the binary's read-only data section, which is
exactly why string literals (log messages, error strings) are so cheap
to use freely in embedded code even with no heap configured.

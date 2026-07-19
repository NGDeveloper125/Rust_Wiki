---
title: "Escape sequences"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [string-literal, char-literal, byte-literal]
see_also: [string-literal, char-literal]
---

## Explanation

Inside a (non-raw) string, character, or byte literal, a backslash
introduces an escape sequence:

- **Quote escapes:** `\'`, `\"`
- **Common ASCII escapes:** `\n` (newline), `\r` (carriage return),
  `\t` (tab), `\\` (backslash), `\0` (null)
- **Numeric byte escape:** `\xNN` — two hex digits; in a `char`/`str`
  context restricted to `\x00`–`\x7F` (7-bit, since a byte escape above
  that isn't a valid standalone Unicode scalar), but a full 8-bit
  `\x00`–`\xFF` in a `b'...'`/`b"..."` byte context
- **Unicode escape:** `\u{7FFF}` — up to six hex digits in braces,
  representing any Unicode scalar value; only legal in `char`/`str`
  contexts, not in byte literals
- **Line-continuation escape:** a backslash immediately followed by a
  newline strips the newline and any leading whitespace on the next line,
  letting a long string literal be wrapped across source lines without
  embedding the line break itself

None of these are processed inside a **raw** string/byte-string literal —
see [raw string literal](raw-string-literal.md).

## Basic usage example

```
let s = "tab\tnewline\n";
//          ^^ ^^ escape sequences, processed at compile time
```

**Restriction:** a `\xNN` byte escape inside a `char`/`str` context is
limited to `\x00`–`\x7F` (7-bit) — the full 8-bit range `\x00`–`\xFF` is
only legal inside a byte (`b'...'`/`b"..."`) literal.

## Best practices & deeper information

### Scenario: Working with text

Embedding `\n`/`\t` directly inside one string literal keeps a short
multi-line template as a single readable value.

```
fn format_receipt(item: &str, qty: u32, price_cents: u32) -> String {
    format!("{item}\n  qty: {qty}\n  price: ${price_cents}\n") // <- `\n` escape sequences: line breaks in one literal
}

print!("{}", format_receipt("widget", 3, 499));
```

**Why this way:** embedding `\n`/`\t` inside one literal keeps the
template readable as "the shape of the output" in one place, which the
[std string docs](https://doc.rust-lang.org/std/string/index.html) treat
as the normal way to write a short template like this.

### Scenario: Serializing and deserializing

Rust's compile-time escape rules and JSON's runtime string-escaping rules
look similar but aren't identical — worth knowing before hand-building
any JSON text.

```
// Rust's `\u{...}` escape (braced, variable-length hex) is *literal* syntax,
// resolved by rustc at compile time -- it is not the same thing as JSON's escape.
let heart: char = '\u{2764}'; // <- Rust escape sequence: braced hex, resolved when this file compiles

// JSON escapes the same code point as `❤` -- four hex digits, no braces --
// and that escaping is *data*, checked by a JSON parser at runtime, not by rustc.
let json_fragment = r#"{"symbol": "❤"}"#; // raw string: the `❤` here is JSON syntax, untouched by Rust
```

**Why this way:** Rust's escape rules and JSON's string-escape rules
differ in small but real ways (braced vs unbraced `\u`, no `\x` byte
escape in JSON at all) — the practical consequence is to let a JSON
library like [serde_json](https://serde.rs/) own all JSON escaping rather
than hand-building JSON text with Rust string literals.

## Embedded Rust Notes

**Full support.** Pure lexical processing, resolved entirely at compile
time — no `std` dependency.

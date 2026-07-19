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

## Embedded Rust Notes

**Full support.** Pure lexical processing, resolved entirely at compile
time — no `std` dependency.

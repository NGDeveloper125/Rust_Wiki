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

## Embedded Rust Notes

**Full support.** A `&str`/`&'static str` literal needs no allocator at
all — it's baked into the binary's read-only data section, which is
exactly why string literals (log messages, error strings) are so cheap
to use freely in embedded code even with no heap configured.

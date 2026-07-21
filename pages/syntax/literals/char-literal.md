---
title: "Character literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [byte-literal, escape-sequences]
see_also: [byte-literal]
---

## Explanation

A single-quoted character literal produces a `char` — a full Unicode
scalar value, always 4 bytes, not a single byte — so `'🦀'` is just as
valid a `char` literal as `'H'`, despite spanning four bytes in UTF-8.

This is a common surprise for newcomers: `char` in Rust is not "one byte"
the way `char` is in C — it represents any Unicode scalar value
(excluding surrogate-pair halves), which is why iterating a `String`
byte-by-byte and iterating it `char`-by-char (`.chars()`) can give very
different results for non-ASCII text. See
[byte literal](byte-literal.md) for the ASCII-only, single-byte
equivalent (`b'H'`).

## Usage examples

### Producing a full Unicode char

```
let grade: char = 'A'; // <- char literal: produces a `char`, a full Unicode scalar value
```

A char literal must contain exactly one Unicode scalar
value — `'ab'` is a compile error, and lone surrogate-pair halves are
never valid scalar values.

### Branching on data (pattern matching)

A small hand-written tokenizer matches individual characters directly,
including as inclusive range-pattern bounds.

```
enum Token {
    Plus,
    Minus,
    Number(char),
    Whitespace,
}

fn classify(c: char) -> Token {
    match c {
        '+' => Token::Plus,             // <- char literal: matched directly as a pattern
        '-' => Token::Minus,            // <- char literal: matched directly as a pattern
        '0'..='9' => Token::Number(c),  // <- char literals as inclusive range-pattern bounds
        ' ' | '\t' => Token::Whitespace,
        _ => panic!("unexpected character {c:?}"),
    }
}
```

Matching `char` literals directly, including as range
bounds, is exactly the form a small tokenizer needs, and the compiler
enforces exhaustiveness over the match — a guarantee spelled out in the
[Reference's pattern grammar](https://doc.rust-lang.org/reference/patterns.html).

### Working with text

A `char` is a decoded Unicode scalar value, not a byte — iterating text
`char`-by-char avoids the panics that raw byte indexing risks on
multi-byte characters.

```
let word = "café";

// PREFER: iterate by char, not by byte, when the text may contain non-ASCII
let first_char: char = word.chars().next().unwrap(); // <- char, not a byte: always a full Unicode scalar value
assert_eq!(first_char, 'c');

let len_chars = word.chars().count(); // 4 chars
let len_bytes = word.len();           // 5 bytes -- 'é' is 2 bytes in UTF-8
assert_ne!(len_chars, len_bytes);
```

A `char` is always a full Unicode scalar value, so
`.chars()` is the safe way to walk text that might contain multi-byte
characters — a `str` can't be indexed by a single `usize` at all, and
*slicing* it (`&s[0..n]`) panics if a bound falls inside a multi-byte
character, as the
[std `str` docs](https://doc.rust-lang.org/std/primitive.str.html#method.get)
note (use `.get()` or char-based iteration instead).

## Explanation (Embedded)

`char` behaves identically under `#![no_std]` — it's a `core` primitive
(`core::char`, re-exported as `std::char` on hosted targets), so nothing
about a `char` literal changes on a bare-metal target: no allocator
involvement, no feature gate, the full Unicode-scalar-value semantics
exactly as on desktop. There isn't a genuinely new embedded-specific
angle to this literal form beyond what the classic explanation already
covers — the one practical note worth repeating in an embedded context
is the same one already given for hosted code: a `char` is always 4
bytes, so on a byte-oriented, ASCII-only interface (UART, AT commands, a
text sensor protocol) a raw `u8`/[byte literal](byte-literal.md) is
usually the more natural fit than decoding to `char` at all.

## Usage examples (Embedded)

### Matching a single command byte from a UART receive buffer

```
fn handle_command(byte: u8) -> &'static str {
    match byte as char {
        'r' => "reset requested",       // <- char literal: matched after casting an incoming UART byte to `char`
        's' => "status requested",
        '\r' | '\n' => "end of line",
        _ => "unknown command",
    }
}
```

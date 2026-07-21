---
title: "// (line comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [block-comment, outer-line-doc-comment]
see_also: [block-comment]
---

## Explanation

`//` begins a comment that runs to the end of the line. It has zero
effect on compilation — the compiler discards it entirely before parsing
(unlike doc comments, which are collected into documentation) — and it
can either sit on its own line or trail after code on the same line.

Nesting doesn't apply, since a line comment simply consumes the rest of
the line regardless of what characters follow.

## Usage examples

### Standalone and trailing comments

```
fn main() {
    // <- this is a line comment: everything from `//` to the end of the line
    let x = 5; // comments can also trail code on the same line
    println!("{x}");
}
```

### Testing

A `//` comment above a non-obvious test case records *why* that specific
input is being checked, so a future reader (including yourself in six
months) doesn't have to reverse-engineer the intent from the assertion
alone.

```
#[test]
fn rejects_empty_username() {
    // regression test for #142: an empty string used to pass validation
    // <- this comment survives in source but never reaches compiled output
    let result = validate_username("");
    assert!(result.is_err());
}
```

The comment explains *why* the test exists, not *what*
the code does — the assertion already says what; only the ticket/context
behind an easy-to-delete-looking test case is worth spelling out.

### Documenting an API

`//` and `///` look almost identical but serve opposite audiences: `//`
is for the next person reading the source, `///` is for the next person
*calling* the function who may never open the source at all.

```
pub struct ParseError;

/// Parses a duration string like "5s" or "10m" into seconds.
pub fn parse_duration(input: &str) -> Result<u64, ParseError> {
    // AVOID: burying caller-relevant info in a // comment nobody sees
    // the trailing unit character determines the multiplier
    let (digits, unit) = input.split_at(input.len() - 1);
    let n: u64 = digits.parse().map_err(|_| ParseError)?;
    match unit {
        "s" => Ok(n),
        "m" => Ok(n * 60),
        _ => Err(ParseError),
    }
}
```

Anything the caller needs to know (accepted formats,
error conditions, examples) belongs in a `///` doc comment — see
[`///`](outer-line-doc-comment.md) — where `cargo doc` and IDE tooltips
surface it. By universal community convention, `//` is for notes aimed at
maintainers reading the source, since it reaches nobody else.

## Explanation (Embedded)

`//` works identically in embedded Rust: it is a lexical construct the
compiler strips before parsing, so it has no dependency on `std` and no
cost on a target with no OS at all. In firmware, `//` is where the
non-obvious hardware facts live — the ones a register name or a HAL call
can't say on its own, like *why* a delay value or bit pattern is what it
is.

## Usage examples (Embedded)

### Documenting a register access

A raw register write is only meaningful with the datasheet fact that
justifies it — the kind of context that belongs beside the code, not in a
public API doc.

```
#![no_std]

const GPIOA_BASE: u32 = 0x4001_0800;
const ODR_OFFSET: u32 = 0x14;

unsafe fn set_pin_5(gpioa: *mut u32) {
    // <- datasheet ref: RM0090 §8.4.6 — ODR bit 5 drives PA5 output high
    let odr = gpioa.byte_add(ODR_OFFSET as usize);
    odr.write_volatile(odr.read_volatile() | (1 << 5));
}
```

### Explaining a timing constant

```
// the sensor's datasheet requires >= 2ms after power-on before the first
// read; 5ms is used here for margin against clock-source startup jitter
const POWER_ON_DELAY_MS: u32 = 5;
```

`//` carries exactly the same weight here as in hosted code — a note for
the next firmware engineer, never emitted into the compiled binary — but
in embedded code that next reader is more often cross-referencing a
datasheet than reading API docs, so the comment tends to cite one.

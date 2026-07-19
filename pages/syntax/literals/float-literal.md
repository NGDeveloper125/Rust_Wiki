---
title: "Floating-point literal"
kind: literal
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [float-suffixes, integer-decimal]
see_also: [float-suffixes]
---

## Explanation

A floating-point literal requires a decimal point, an exponent, or a type
suffix — otherwise it's parsed as an integer:

```
let a = 1.0;
let b = 0.1;
let c = 123.0E+2;
let d = 2.;      // trailing decimal point alone is enough
```

Without a suffix, the default type is `f64`, not `f32` — the opposite
default from many other languages. A literal like `2.` (decimal point,
no digits after) is legal but `2.method()` is ambiguous with method-call
syntax, so a space or parentheses (`2. .abs()` or `(2.).abs()`) is
sometimes required to disambiguate.

## Basic usage example

```
let temp = 36.6; // <- float literal: the decimal point makes this an `f64` by default
```

**Restriction:** a bare trailing-dot literal like `2.` is ambiguous with
method-call syntax immediately after — `2.abs()` requires a space or
parentheses (`2 .abs()` / `(2.).abs()`) to parse as intended.

## Embedded Rust Notes

**Full support**, but worth checking your target: many microcontrollers
have no hardware floating-point unit, so `f32`/`f64` arithmetic compiles
to (much slower) software emulation routines. Fixed-point integer
arithmetic is a common alternative on FPU-less targets.

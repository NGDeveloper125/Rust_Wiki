---
title: ">>="
kind: operator
embedded_support: full
groups: [Basics]
related_concepts: [Operator overloading]
related_syntax: [">>"]
see_also: [">>"]
---

## Explanation

`>>=` right-shifts the left operand by the right operand's amount, in
place, overloadable via `std::ops::ShrAssign`.

```
let mut x = 8u8;
x >>= 3; // x is now 1
```

## Basic usage example

```
let mut x = 8u8;
x >>= 3; // <- `>>=` right-shifts `x` in place
```

## Best practices & deeper information

### Scenario: Bit manipulation and flags

Unpacking a byte's fields one at a time by repeatedly consuming its
lowest bits is a natural use of `>>=` — each pass shifts the next field
into position while permanently discarding the bits already read.

```
let mut packed: u8 = 0b1011_0110; // 3 fields: 2 + 3 + 3 bits

let field_a = packed & 0b11;
packed >>= 2; // <- `>>=` discards the consumed 2 bits, shifts the rest down
let field_b = packed & 0b111;
packed >>= 3; // <- `>>=` consumes the next field the same way
let field_c = packed & 0b111;

assert_eq!((field_a, field_b, field_c), (0b10, 0b101, 0b101));
```

**Why this way:** mutating `packed` in place with `>>=` as each field is
consumed avoids tracking a separate shift-amount variable and re-deriving
it every time — the same in-place rationale as
[`+=`](plus-equals.md), here specialized to `ShrAssign`.

## Embedded Rust Notes

**Full support.** `ShrAssign` lives in `core::ops` — no `std` dependency.

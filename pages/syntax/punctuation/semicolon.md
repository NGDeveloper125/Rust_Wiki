---
title: ";"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language]
related_syntax: ["{ }"]
see_also: ["{ }"]
---

## Explanation

`;` terminates a statement, and — just as importantly in an
expression-oriented language — turns what would otherwise be an
expression into a statement that evaluates to `()`:

```
let x = 5;      // statement
x + 1;          // expression, discarded, statement evaluates to ()
x + 1           // expression, this IS the value (no semicolon)
```

This is why leaving off the trailing semicolon on a block's last line
means "return this value," while adding one means "run this and discard
the result." Getting this wrong (an accidental trailing `;`) is one of the
most common beginner mistakes — a function meant to return a value
silently returns `()` instead, usually caught only by a type error at the
call site.

`;` also appears inside `[Type; N]` (array type/literal — see
[square brackets](square-brackets.md)), which is unrelated to its
statement-terminator role.

## Embedded Rust Notes

**Full support.** Pure statement-terminator grammar — no `std` dependency.

---
title: "/* */ (block comment)"
kind: comment
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: [line-comment, outer-block-doc-comment]
see_also: [line-comment]
---

## Explanation

`/* ... */` comments out everything between the delimiters, including
line breaks:

```
/* this whole
   block is ignored */
let x = 5;
```

Unlike C, Rust block comments **nest**: `/* outer /* inner */ still outer */`
is a single, correctly-closed comment — the compiler tracks nesting depth
rather than closing at the first `*/` encountered. This makes it safe to
comment out a chunk of code that itself already contains a block comment.

## Basic usage example

```
fn main() {
    /* <- this is a block comment: everything up to the matching `*/` is ignored,
       even across multiple lines */
    let x = 5;
    /* nesting works: /* an inner comment */ doesn't end the outer one early */
    println!("{x}");
}
```

**Restriction:** the opening `/*` and closing `*/` must both be present —
an unterminated block comment is a compile error, unlike a line comment
which simply ends at the newline.

## Embedded Rust Notes

**Full support.** Pure lexical construct — no `std` dependency.

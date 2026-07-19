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

## Embedded Rust Notes

**Full support.** Pure lexical construct — no `std` dependency.

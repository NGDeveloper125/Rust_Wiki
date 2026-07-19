---
title: "|"
kind: operator
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching", "Functions & Closures"]
related_concepts: [Operator overloading, "Closures & capturing", "Pattern matching"]
related_syntax: ["||", "&", "|="]
see_also: ["||"]
---

## Explanation

`|` has three unrelated meanings depending on context:

1. **Binary: bitwise OR** between integers, overloadable via
   `std::ops::BitOr`: `a | b`.
2. **Pattern alternatives:** `Some(1) | Some(2) => ...` inside a `match`
   arm (or any refutable pattern position) — "matches if any of these
   patterns match." Entirely unrelated to bitwise OR; no `BitOr` impl is
   involved.
3. **Closure parameter delimiters:** `|x, y| x + y` — opens and closes
   the closure's parameter list, standing in for the parentheses a
   function's parameter list would use.

The empty-parameter-list closure form uses `||` (its own token — see
[`||`](pipe-pipe.md)) rather than `| |` with a space; the two are not
interchangeable in the grammar.

## Embedded Rust Notes

**Full support** for all three meanings. `BitOr` lives in `core::ops` —
setting flag bits in a hardware register (`reg | ENABLE_BIT`) is a
routine embedded operation; pattern alternatives and closure delimiters
are pure grammar either way.

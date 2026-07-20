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
line breaks — `/* this whole block is ignored */` works the same whether
it stays on one line or spans several.

Unlike C, Rust block comments **nest**: `/* outer /* inner */ still outer */`
is a single, correctly-closed comment — the compiler tracks nesting depth
rather than closing at the first `*/` encountered. This makes it safe to
comment out a chunk of code that itself already contains a block comment.

## Basic usage example

```
fn main() {
    /* <- this is a block comment: everything up to the matching
       closing delimiter is ignored, even across multiple lines */
    let x = 5;
    /* nesting works: /* an inner comment */ doesn't end the outer one early */
    println!("{x}");
}
```

**Restriction:** the opening `/*` and closing `*/` must both be present —
an unterminated block comment is a compile error, unlike a line comment
which simply ends at the newline.

## Best practices & deeper information

### Scenario: Testing

While tracking down a failing test, it's common to temporarily comment
out a whole test function to isolate the problem. `/* */`'s nesting is
what makes this safe even when the test body already contains its own
comments — a plain `//`-based approach would require commenting out
every line individually.

```
/*
#[test]
fn flaky_retry_logic() {
    // this test intermittently fails on slow CI runners — disabled
    // while investigating; see issue tracker
    let result = retry_with_backoff(3);
    assert!(result.is_ok());
}
*/
// <- the whole block above (including its own // comments) is inert;
//    because /* */ nests, any *balanced* inner /* ... */ pair in the
//    disabled code can't accidentally close this wrapper early

#[test]
fn stable_retry_logic() {
    assert_eq!(retry_with_backoff(0), Ok(()));
}
```

Note the limit of the nesting guarantee: an *unmatched* stray `*/` in the
disabled code (say, inside a string literal) still closes the wrapper at
that point — nesting only protects properly paired inner comments.

**Why this way:** this is a deliberately temporary debugging aid, not a
substitute for `#[ignore]` — once the investigation is done, either fix
the test or mark it properly with `#[ignore = "reason"]` so it still
shows up (as skipped) in `cargo test` output instead of silently
vanishing from the codebase.

## Embedded Rust Notes

**Full support.** Pure lexical construct — no `std` dependency.

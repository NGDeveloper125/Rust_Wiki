---
title: "|"
kind: operator
embedded_support: full
groups: [Bitwise, Basics, "Control Flow & Pattern Matching", "Functions & Closures"]
related_concepts: [Operator overloading, "Closures & capturing", "match expressions"]
related_syntax: ["||", "&", "|="]
see_also: ["||"]
---

## Explanation

`|` has three unrelated meanings depending on context:

1. **Binary: bitwise OR** between integers, overloadable via
   `std::ops::BitOr`: `a | b` (`BitOr` is also implemented for `bool`,
   making `a | b` a non-short-circuiting logical OR).
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

## Usage examples

### Bitwise OR between two integers

```
let mask = 0b0100 | 0b0001; // <- `|` bitwise OR between two integers
```

### Bit manipulation and flags

Combining several flag bits into one value to pass around or compare
against is the bitwise-OR use of `|` — distinct from `|=`, which mutates
an existing binding rather than producing a new combined value.

```
const FLAG_READY: u8   = 0b0000_0001;
const FLAG_LOGGING: u8 = 0b0000_0100;

fn startup_flags(verbose: bool) -> u8 {
    if verbose {
        FLAG_READY | FLAG_LOGGING // <- `|` combines two bits into one value
    } else {
        FLAG_READY
    }
}

assert_eq!(startup_flags(true), 0b0000_0101);
```

Building the combined value with `|` and returning it
(rather than mutating a `mut` accumulator with `|=`) fits a function
that hands back a fresh flag set rather than modifying state in place —
see [`|=`](pipe-equals.md) for the in-place variant of the same bitwise
operation.

### Branching on data (pattern matching)

Inside a `match` arm, `|` separates alternative patterns that should all
take the same branch — a different meaning of the same token from
bitwise OR, resolved entirely by position (pattern position vs.
expression position).

```
enum HttpStatus {
    Ok,
    Created,
    NotFound,
    ServerError(u16),
}

fn is_client_error(status: &HttpStatus) -> bool {
    match status {
        HttpStatus::NotFound => true,
        HttpStatus::Ok | HttpStatus::Created => false, // <- `|`: matches either variant
        HttpStatus::ServerError(_) => false,
    }
}
```

An or-pattern collapses what would otherwise be two
near-identical match arms with the same body, which the
[Rust Reference](https://doc.rust-lang.org/reference/patterns.html#or-patterns)
documents as a first-class pattern form rather than sugar layered on top
of separate arms.

## Embedded Rust Notes

**Full support** for all three meanings. `BitOr` lives in `core::ops` —
setting flag bits in a hardware register (`reg | ENABLE_BIT`) is a
routine embedded operation; pattern alternatives and closure delimiters
are pure grammar either way.

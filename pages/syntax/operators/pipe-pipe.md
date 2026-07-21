---
title: "||"
kind: operator
embedded_support: full
groups: [Logical, Basics, "Functions & Closures"]
related_concepts: []
related_syntax: ["&&", "|"]
see_also: ["&&"]
---

## Explanation

`||` is short-circuiting logical OR between two `bool` values, as in
`if a < 0 || b < 0 { ... }` — the right operand only evaluates if the
left is `false`. Like `&&`, `||` is not overloadable — always `bool`,
always short-circuiting.

`||` also opens and closes a **zero-argument closure**'s parameter list,
as in `let f = || println!("called");`. This is the same `|...|` closure
syntax as `|x, y| x + y`, just with an
empty parameter list — the parser distinguishes the two uses by position:
`||` with an expression on its left is lazy OR, while `||` at the start
of an expression is a closure's (empty) parameter list.

## Usage examples

### Short-circuiting logical OR between two conditions

```
let a = 150;
let out_of_range = a < 0 || a > 100; // <- `||` short-circuiting logical OR
```

### Validating input

Rejecting a request that's either empty or over a configured limit reads
naturally as an `||` of two independent conditions, with short-circuiting
avoiding a wasted `len()` call when the list is already empty.

```
const MAX_ITEMS: usize = 100;

fn reject(order_items: &[u32]) -> bool {
    order_items.is_empty() || order_items.len() > MAX_ITEMS
    //                     ^^ only evaluated if `is_empty()` was false
}

assert!(reject(&[]));
assert!(!reject(&[1, 2, 3]));
```

Ordering the cheaper, more-likely-true check first
lets short-circuiting skip the second check entirely on the common path,
which the [Rust Reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html#lazy-boolean-operators)
documents as guaranteed evaluation order, not just an optimization detail
to rely on incidentally.

### Handling and propagating errors

Guarding a fallible operation behind an `||` check lets the guard clause
bail out via `?` before an operation that would otherwise panic or return
an `Err`, such as indexing into a possibly-too-short buffer.

```
const HEADER_MARKER: u8 = 0x00;

fn read_header(buf: &[u8]) -> Result<u16, &'static str> {
    if buf.len() < 2 || buf[0] != HEADER_MARKER {
        // <- `||`: either condition alone is enough to reject `buf`
        return Err("missing or malformed header");
    }
    Ok(u16::from_be_bytes([buf[0], buf[1]]))
}

assert!(read_header(&[0x01]).is_err());
assert_eq!(read_header(&[0x00, 0x2A]), Ok(42));
```

Checking both failure conditions up front with `||`
turns a potential panic (indexing into a too-short slice) into a handled
`Err` — and short-circuiting means `buf[0]` on the right is only ever
evaluated once the length check on the left has passed, in line with the
fail-fast validation style the
[Book](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)
recommends for recoverable input errors.

## Embedded Rust Notes

**Full support** for both meanings — logical OR and zero-argument
closures both work identically in `#![no_std]` (closures don't require
heap allocation unless they're boxed as `dyn Fn`).

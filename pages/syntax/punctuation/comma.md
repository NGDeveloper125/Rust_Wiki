---
title: ","
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: []
related_syntax: ["( )", "[ ]"]
see_also: ["( )"]
---

## Explanation

`,` separates elements in any list: function arguments (`f(a, b, c)`),
tuple elements (`(a, b, c)`), array/`Vec` elements (`[a, b, c]`), struct
fields (`Point { x, y }`), enum variant fields, generic parameters
(`Vec<K, V>`), and match arms in some macro contexts.

A trailing comma after the last element is allowed (and idiomatic in
multi-line lists — `rustfmt` adds it automatically) in every one of these
positions:

```
let v = vec![
    1,
    2,
    3,
];
```

The one place a single trailing comma is *required*, not just allowed, is
a one-element tuple: `(x,)` — without the comma, `(x)` is just a
parenthesized expression, not a tuple at all.

## Basic usage example

```
let point = (1, 2, 3);
//            ^     ^ each `,` separates one tuple element from the next
```

**Restriction:** in a one-element tuple, the trailing comma is
*mandatory* — `(x,)` is a tuple, `(x)` is just `x` in parentheses.

## Best practices & deeper information

### Scenario: Working with collections

Multi-line collection literals read best with one element per line and a
trailing comma on the last one — `rustfmt`'s default output — so adding a
new element is a one-line diff instead of touching the previous line too.

```
let allowed_hosts = vec![
    "api.example.com",
    "cdn.example.com",
    "auth.example.com", // <- trailing comma: adding a 4th host below stays a 1-line diff
];
```

**Why this way:** without the trailing comma, inserting a new element
means editing the *previous* line to add a comma too, which shows up as
a noisy, unrelated change in a diff/code review — `rustfmt` inserts
trailing commas in multi-line lists specifically to keep diffs minimal.

### Scenario: Creating a new object

Struct-literal fields are comma-separated exactly like any other list —
including the same trailing-comma convention once the literal spans
multiple lines.

```
struct Config {
    host: String,
    port: u16,
    timeout_secs: u32,
}

let cfg = Config {
    host: "localhost".into(),
    port: 8080,
    timeout_secs: 30, // <- trailing comma on the last field, rustfmt's default
};
```

**Why this way:** consistent trailing commas across struct literals,
function calls, and collection literals mean `rustfmt` never has to
special-case one construct — one rule, applied everywhere reduces the
number of style decisions a codebase has to make.

## Embedded Rust Notes

**Full support.** Pure list-separator grammar — no `std` dependency.

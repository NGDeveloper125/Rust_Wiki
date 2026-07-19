---
title: "{ }"
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Expression-oriented language, Structs]
related_syntax: [";"]
see_also: [";"]
---

## Explanation

`{ }` delimits a **block expression** — a sequence of statements followed
by an optional final expression:

```
let y = {
    let x = 1;
    x + 1
};
```

A block is itself an expression: it evaluates to its final expression (if
it has no trailing `;`), or to `()` otherwise. Function bodies, `if`/`else`
arms, loop bodies, and match arms are all block expressions under the
hood, which is exactly why `if`/`match` can produce values.

`{ }` is reused for a completely different purpose in `Type { field: value, ... }`
— a **struct literal** — where it delimits field initializers rather than
statements. The two uses are distinguished purely by what precedes the
brace (a type path vs. nothing), which is also why `if SomeStruct { .. } { }`
needs disambiguating parentheses in condition position — the parser would
otherwise try to read the struct literal as the `if`'s block.

## Basic usage example

```
let y = { // <- `{` opens a block expression
    let x = 1;
    x + 1 // no trailing `;`, so this is the block's value
}; // <- `}` closes it; y is now 2
```

## Best practices & deeper information

### Scenario: Creating a new object

A struct literal's `{ }` can build the whole value in one expression,
including computing derived fields inline — no separate mutation step
needed afterward.

```
struct Rectangle { width: f64, height: f64, area: f64 }

fn rectangle(width: f64, height: f64) -> Rectangle {
    Rectangle { width, height, area: width * height } // <- `{ }` builds the whole value at once
}
```

**Why this way:** constructing the fully-formed value in one struct
literal, rather than creating a default/partial value and mutating fields
into place, avoids ever having an inconsistent intermediate state (e.g.
`area` not yet matching `width`/`height`) that some other code could
observe.

### Scenario: Branching on data (pattern matching)

Every `match` arm's body is itself a block expression — `{ }` is what
lets an arm run several statements before producing its value, while a
short arm can skip the braces entirely.

```
let description = match status {
    Status::Ok => "ready",
    Status::Error(code) => { // <- `{` opens a multi-statement arm body
        log::warn!("request failed with code {code}");
        "failed"
    } // <- `}` closes it; "failed" is this arm's value
};
```

**Why this way:** because every arm is a block expression, all arms must
produce the same type — this is what lets `match` be used as an
expression assigned directly to a binding, rather than requiring a
separate mutable variable set inside each arm.

## Embedded Rust Notes

**Full support.** Block and struct-literal delimiters are core grammar —
no `std` dependency.

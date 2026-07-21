---
title: "if let"
kind: keyword
embedded_support: full
groups: ["Control Flow", Basics, "Control Flow & Pattern Matching"]
related_concepts: ["if let / while let"]
related_syntax: [if, else, let, match]
see_also: [if, else, match]
---

## Explanation

`if let PATTERN = EXPR { ... }` tests whether `EXPR` matches `PATTERN`. If it
does, any names `PATTERN` introduces are bound within the following block. It
differs from a plain [`if`](if.md) in what's being tested: a plain `if`
requires a `bool` condition, while `if let` succeeds or fails based on
whether a pattern matches — the two are not interchangeable, and `if let`
exists precisely for patterns that don't reduce to a single boolean. See
[if let / while let](../../concepts/pattern-matching/if-let-and-while-let.md)
for the deeper "why" — when this lighter form is preferable to a full
`match`.

An `if let` can be followed by further pattern tests chained with
`else if let PATTERN_2 = EXPR_2 { ... }`, and a final plain `else { ... }`
for whatever matched none of the preceding patterns — the basic usage
example below shows the full three-part chain.

This is the same "just an `if` nested inside the previous `else`" shape
described on the [`else`](else.md) page, extended here to pattern tests
instead of boolean conditions — each `else if let` tries a new pattern
(possibly against a different expression) only once the ones before it have
failed.

**`if let`/`else` vs. `let else`:** these look related but solve different
problems. `if let PATTERN = EXPR { A } else { B }` produces two branches, `A`
and `B`, that both continue executing normally afterward — the pattern's
success or failure only decides which of two live paths runs, and code
after the whole construct is reached from either one. `let PATTERN = EXPR
else { ... };` (see [`else`](else.md) for its full treatment) is shaped
differently: it isn't a fork with two continuing paths, it's a single
binding that either succeeds — falling through to the rest of the
enclosing scope with `PATTERN`'s names in scope — or fails and runs the
`else` block, which is *required* to diverge (`return`, `break`, `continue`,
`panic!`, or similar). There is no third option where `let else`'s `else`
block finishes normally and execution continues past the `let`, because the
rest of the scope depends on `PATTERN`'s bindings having been established.
Reach for `if let`/`else` when both outcomes have real, continuing code to
run; reach for `let else` when the non-matching case is a bail-out and the
matching case is meant to keep going with the bound value in scope.

Like `if`, an `if let`/`else` chain is an expression when every branch
produces the same type — it can sit on the right of a `let`, though this is
less common than using it purely for its side effects.

## Basic usage example

```
enum ConfigValue {
    Number(i64),
    Text(String),
    Missing,
}

let value = ConfigValue::Text("info".to_string());

if let ConfigValue::Number(n) = &value {
    // <- `if let` tests the pattern against `&value`
    println!("number: {n}");
} else if let ConfigValue::Text(s) = &value {
    // <- `else if let`: a second, different pattern tried only if the first failed
    println!("text: {s}");
} else {
    println!("missing"); // <- runs only if neither pattern above matched
}
```

## Best practices & deeper information

### Scenario: Branching on data (pattern matching)

A network frame handler needs different handling for a data payload versus
a close frame, with anything else treated as a keepalive — `if let`/
`else if let` walks through the shapes it actually cares about.

```
enum Frame {
    Data(Vec<u8>),
    Close(u16),
    Ping,
}

fn describe(frame: &Frame) -> String {
    if let Frame::Data(bytes) = frame {
        // <- `if let` matches the Data shape and binds `bytes`
        format!("{} bytes of data", bytes.len())
    } else if let Frame::Close(code) = frame {
        // <- `else if let` tries the Close shape next
        format!("closing with code {code}")
    } else {
        "ping".to_string()
    }
}
```

**Why this way:** each pattern is tried in order until one matches, which
reads as "which of these shapes is it" without the ceremony of a `match`'s
arms for a case this small — the
[Rust Book](https://doc.rust-lang.org/book/ch06-03-concise-control-flow-with-if-let-and-let-else.html)
introduces `if let` for exactly this "only some shapes need code" situation.

### Scenario: Handling and propagating errors

Reporting a metrics data point is best-effort: the caller only wants to log
a failure, not stop the program over it — `if let Err(...)` isolates just
that one case.

```
fn report_metric(result: Result<(), String>) {
    if let Err(reason) = result {
        // <- only the failure case needs a reaction; success falls through silently
        eprintln!("metric send failed: {reason}");
    }
}
```

**Why this way:** when the success case genuinely needs no follow-up, `if
let Err(...)` avoids writing an `Ok(()) => {}` arm that would say nothing —
the same "only one shape matters here" reasoning the
[Rust Book](https://doc.rust-lang.org/book/ch06-03-concise-control-flow-with-if-let-and-let-else.html)
gives for preferring `if let` over a full `match` when only one outcome
needs handling.

### Scenario: Working with collections

Looking up a product's stock count only has something useful to say when
the SKU is known — `if let Some(...)` handles the found case, with `else`
covering an unrecognized SKU.

```
use std::collections::HashMap;

fn describe_stock(inventory: &HashMap<String, u32>, sku: &str) -> String {
    if let Some(&count) = inventory.get(sku) {
        // <- `if let` handles only the "found" case
        format!("{sku}: {count} in stock")
    } else {
        format!("{sku}: unknown SKU")
    }
}
```

**Why this way:** `HashMap::get` returns `Option<&V>` specifically so a
missing key is an ordinary value to match on rather than a panic — `if
let`/`else` gives both outcomes (found, not found) a place to go without
the extra `None => ...` arm a full `match` would need here.

## Embedded Rust Notes

**Full support.** Core-language pattern matching, allocator-free, compiling
to the same code a hand-written `match` would. Commonly used to check the
`Option` a peripheral read function returns without the ceremony of a full
`match` when only the `Some` case needs a reaction.

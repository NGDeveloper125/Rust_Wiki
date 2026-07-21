---
title: "Reserved syntax & edition gotchas"
kind: punctuation
embedded_support: full
groups: ["Reserved Keywords"]
related_concepts: []
related_syntax: ["raw-string-literal", "byte-string-literal", "c-string-literal", "byte-literal", "string-literal"]
see_also: ["raw-string-literal", "byte-string-literal", "c-string-literal"]
---

## Explanation

Some character sequences that look like they should parse one way are
deliberately made illegal instead — not because the underlying tokens are
ambiguous today, but so a **future edition** has room to give that
sequence new meaning without breaking code that edition compiles. These
are "reserved syntax": not a keyword, not an operator, just a shape the
lexer refuses on purpose. Hitting one feels like a bug in your code (a
"reserved" error on something that looks otherwise fine), but it's a
guardrail, not a mistake — most often surfaced by a macro that pastes
tokens together and accidentally produces one of these shapes.

**Reserved prefixes (2021 edition and later).** Writing an identifier
directly against `#`, `'`, or `"` with **no space** — `ident#`, `ident'`,
`ident"..."` — is reserved syntax starting in the 2021 edition, rather
than silently being lexed as two separate tokens (the identifier, then
the `#`/`'`/`"` starting its own token) the way earlier editions allowed.
For example, `z"hey"` used to lex as the identifier `z` immediately
followed by the string literal `"hey"` — two tokens, whatever came next
being the code's problem to sort out. Since 2021, that exact adjacency
is rejected outright as reserved syntax instead.

The reason is future-proofing: it leaves room for **new prefixed-literal
forms** later — something like a hypothetical `k"..."` for a
kilometers-flavored literal, or any other `letters"..."` shape — to be
introduced in a future edition without it being a backwards-incompatible
change. If `ident"..."` already had an established two-token meaning
today, claiming it for a new prefixed literal tomorrow would silently
change the meaning of existing code. Reserving the shape now, before
anything depends on the old two-token reading, means a future edition
can hand it a new meaning for free.

This reservation has explicit carve-outs for prefixed forms that
**already exist** and remain perfectly legal: `b'a'` (a
[byte literal](../literals/byte-literal.md)), and the string forms
`b"..."` ([byte string](../literals/byte-string-literal.md)), `c"..."`
([C string](../literals/c-string-literal.md)), `r"..."`
([raw string](../literals/raw-string-literal.md)), `br"..."` (raw byte
string), and `cr"..."` (raw C string) — all still work exactly as
before. What's reserved is any *other* letter sequence directly against
a quote or `#`, not those specific, already-claimed prefixes.

**Reserved string guards (2024 edition and later).** Separately, `#"` at
the start of a string-like literal and a bare `##` are reserved starting
in the 2024 edition. This is held for a possible future alternate
string-literal delimiter — something in the spirit of extending raw
strings' `#`-fencing (`r#"..."#`, `r##"..."##`) further, giving a future
edition a new delimiter scheme to introduce without colliding with
existing code.

In both cases, the fix when you hit one of these errors is almost always
mechanical: add a space (or another separator) between the identifier
and the following `#`/`'`/`"`, or restructure macro-generated code so it
doesn't paste an identifier directly against one of these characters.

## Basic usage example

```
// 2021 edition and later:
let z = 1;
// z"hey";   // error: prefix `z` is unknown (reserved syntax)
let ok = "hey"; // fine: no adjacency between an identifier and a quote

// 2024 edition and later:
// #"literal"   // error: invalid string literal (reserved guard syntax)
let ok2 = "literal"; // fine: an ordinary string literal, no leading `#"`
```

## Best practices & deeper information

### Scenario: Working with text

A small templating helper that stitches an identifier-like tag onto a
literal string, generated through a macro, can accidentally produce a
reserved-prefix shape if the macro pastes tokens together with no
separator.

```
macro_rules! log_event {
    ($tag:ident, $message:expr) => {
        println!("[{}] {}", stringify!($tag), $message)
        // AVOID: concat!(stringify!($tag), $message) risks pasting an
        // identifier directly against a string literal if $message ever
        // arrives as a literal token rather than an already-formatted value.
    };
}

log_event!(order_placed, "customer checkout completed");
```

**Why this way:** keeping the identifier and the string as two separate
arguments passed through `println!`/`format!`, rather than concatenating
their *tokens* together at macro-expansion time, avoids ever generating
an `ident"..."` shape the 2021-edition reserved-prefix rule would reject
— the
[2021 edition guide's reserved syntax section](https://doc.rust-lang.org/edition-guide/rust-2021/reserved-syntax.html)
documents this exact prefix-adjacency case.

### Scenario: Designing a public API

A library author who likes the ergonomics of `sql"SELECT ..."`-style
literal prefixes for a domain-specific string (as some other languages
allow) needs to know upfront that this space is walled off, not merely
unfashionable — writing `sql!("SELECT ...")` as an ordinary macro call
is the available alternative today.

```
macro_rules! sql {
    ($query:expr) => {
        $query // a real implementation would validate/parse $query here
    };
}

let query = sql!("SELECT id FROM orders WHERE status = 'shipped'");
// A prefix-literal spelling like sql"SELECT ..." is not available:
// `ident"..."` adjacency is reserved syntax since the 2021 edition,
// specifically so a *future* edition — not a third-party macro — can
// claim new prefixes like this.
```

**Why this way:** the reserved-prefix rule exists precisely to keep
`letters"..."` available for the *language* to standardize later, not
for individual crates to informally claim — designing the public API
around an explicit macro call (`sql!(...)`) rather than chasing a
prefix-literal syntax keeps the crate compatible with whatever a future
edition eventually does with that space, per the
[Rust Reference's tokens chapter](https://doc.rust-lang.org/reference/tokens.html#reserved-prefixes).

## Embedded Rust Notes

**Full support.** Reserved syntax is a lexer-level restriction, identical
in `#![no_std]` and hosted Rust alike.

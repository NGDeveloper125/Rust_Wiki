---
title: "_"
kind: punctuation
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching"]
related_concepts: ["Destructuring", "match expressions", "Exhaustiveness checking"]
related_syntax: [match, let, digit-separator]
see_also: [match, digit-separator]
---

## Explanation

`_` is the wildcard pattern: it matches any value of any shape without
binding it to a name. Three positions cover most of its use:

- **Match arm catch-all:** `_ => expression` matches whatever no earlier
  arm did, satisfying [exhaustiveness](../../concepts/pattern-matching/exhaustiveness-checking.md)
  without writing out every remaining case by hand.
- **Explicit discard in a `let`:** `let _ = might_fail();` runs the
  expression, then immediately drops its value. This differs from binding a
  real (if unused) name — `let _guard = expr;` keeps the value alive for
  the rest of the scope (useful for RAII guards/locks), where `let _ = expr;`
  drops it on the spot. It also differs from writing `expr;` as a bare
  statement: both discard the value immediately, but `let _ = expr;` reads
  as a deliberate choice to ignore the result, while a bare statement gives
  no signal that ignoring it was intentional rather than an oversight —
  and for a `#[must_use]` type such as `Result`, only `let _ =` (not a bare
  statement) reliably suppresses the associated lint.
- **Unused function parameter:** `fn f(_: i32) {}` accepts an argument
  without binding it to a name, so the compiler has nothing to warn about
  as unused. This is common in trait implementations where a method's
  signature requires a parameter the implementation doesn't need.

`_` also appears as a type placeholder for inference (`let v: Vec<_> = ...`),
asking the compiler to fill in a type it can determine from context, rather
than as a pattern at all.

**Not to be confused with:** `_` used *inside a numeric literal*
(`1_000_000`) is a completely different, unrelated token — the
[digit separator](../literals/digit-separator.md). The wildcard pattern on
this page only ever appears where a pattern, binding, or parameter name is
expected; the digit separator only ever appears between the digits of a
number. Same character, disjoint grammars, resolved entirely by where it
sits.

## Usage examples

### Using `_` as a match catch-all

```
let result: Result<i32, String> = Err("disk full".to_string());

match result {
    Ok(value) => println!("got {value}"),
    _ => println!("something went wrong"), // <- `_` matches any value here without binding it
}
```

### Validating input

Not every discarded `Result` is the same: a best-effort cleanup can safely
ignore its outcome, but a genuinely important operation should never be
silenced the same way.

```
fn cleanup_temp_file(path: &str) {
    let _ = std::fs::remove_file(path); // <- `_`: best-effort cleanup, fine to ignore any failure
}

// AVOID: discarding a Result whose failure is exactly what the caller needs to know about
fn submit_order_avoid(order_id: u32) {
    let _ = process_order(order_id); // <- silently masks a failed order as if it succeeded
}

// PREFER: propagate it instead, since a failed submission is not safe to ignore
fn submit_order(order_id: u32) -> Result<(), String> {
    process_order(order_id)
}

fn process_order(order_id: u32) -> Result<(), String> {
    if order_id == 0 { Err("invalid order id".to_string()) } else { Ok(()) }
}
```

`let _ = ...` is Clippy's documented way to intentionally
silence the
[`unused_must_use`](https://rust-lang.github.io/rust-clippy/master/#unused_must_use)
lint on a `Result` you've decided not to check — using it on a value whose
failure genuinely matters just hides the bug instead of fixing it, which is
why `submit_order_avoid` above is a defect, not a stylistic choice.

### Branching on data (pattern matching)

A process exit code only has a handful of well-known meanings; every other
value still needs somewhere to go.

```
fn describe_exit_code(code: i32) -> &'static str {
    match code {
        0 => "success",
        1 => "general error",
        127 => "command not found",
        _ => "unknown exit code", // <- `_` catches every other i32 value
    }
}

println!("{}", describe_exit_code(127));
```

`i32` has far more values than any list of literals could
enumerate, so a `_` catch-all is the only way this compiles at all — the
[Rust Reference on exhaustiveness](https://doc.rust-lang.org/reference/expressions/match-expr.html#match-expressions)
requires every value of the scrutinee's type to be covered by some arm.

## Explanation (Embedded)

`_` means exactly the same thing under `#![no_std]` — wildcard pattern,
discard binding, and unused-parameter marker, all allocator-free with
zero runtime cost. It's especially common as a `match` catch-all when
decoding a memory-mapped register: a status/control register usually
reserves only a handful of bit patterns as meaningful and leaves the rest
undefined, and `_` covers "every other bit pattern" in one arm instead of
enumerating values that should never occur.

## Usage examples (Embedded)

### Catching every other bit pattern in a status register

```
fn decode_status(reg: u8) -> &'static str {
    match reg & 0b0000_0011 {
        0b00 => "idle",
        0b01 => "receiving",
        0b10 => "transmitting",
        _ => "reserved", // <- `_` catches the one remaining 2-bit pattern
    }
}
```

### Discarding a fallible peripheral write

```
fn set_led_high(gpioa: &pac::GPIOA) {
    let _ = gpioa.bsrr.write(|w| w.bs5().set_bit()); // <- `_`: this write can't fail in practice, ok to discard
}
```

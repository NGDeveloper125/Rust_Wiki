---
title: "=>"
kind: punctuation
embedded_support: full
groups: [Basics, "Control Flow & Pattern Matching"]
related_concepts: ["match expressions"]
related_syntax: [match, "->", "|", "@"]
see_also: [match, "->"]
---

## Explanation

`=>` separates a [`match`](../keywords/match.md) arm's pattern from the code
that runs when it matches: `pattern => expression`. It appears only inside a
`match` block's arm list — the arm's full grammar (comma rules, guards,
or-patterns via `|`, bindings via `@`) is covered on the `match` page and
isn't repeated here.

**Disambiguation from `->`:** `=>` (fat arrow) is unrelated to
[`->`](arrow.md) (thin arrow) despite the visual similarity and both
appearing near function-like constructs. `->` introduces a function's or
closure's *return type* (`fn f() -> i32`); `=>` separates a match arm's
*pattern from its body*. Mixing them up is a common slip for newcomers,
especially coming from languages (JavaScript, Kotlin, C#) where `=>` itself
introduces a lambda/arrow-function body. Rust's closures use `|params| body`
instead, with no arrow at all unless an explicit return type is spelled out
via `->`.

`=>` also appears in `macro_rules!` arms, separating a macro's matcher
pattern from its expansion (`(pattern) => { expansion };`). That is a
distinct, macro-specific grammar with its own matching rules — covered on
the macro syntax pages, not here.

## Usage examples

### Separating a match arm's pattern from its result

```
let day = 3;

let name = match day {
    1 => "Monday", // <- `=>` separates the pattern from what runs if it matches
    2 => "Tuesday",
    3 => "Wednesday",
    _ => "unknown",
};

println!("{name}");
```

### Branching on data (pattern matching)

Looking up a playing card's color from its suit is a one-to-one mapping —
each arm's `=>` leads straight to the result, no further computation
needed.

```
enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

fn color(suit: &Suit) -> &'static str {
    match suit {
        Suit::Hearts => "red",  // <- `=>` separates the pattern from its result
        Suit::Diamonds => "red",
        Suit::Clubs => "black",
        Suit::Spades => "black",
    }
}

println!("{}", color(&Suit::Diamonds));
```

A one-line arm body after `=>` keeps a simple mapping
scannable at a glance — the
[Rust Book](https://doc.rust-lang.org/book/ch06-02-match.html) uses this
same terse, single-expression-per-arm style whenever an arm's logic doesn't
need a block.

### Handling and propagating errors

Reporting on a network fetch needs a different message per outcome — here
`=>` leads into a multi-step expression rather than a single literal.

```
fn describe_fetch(result: Result<String, String>) -> String {
    match result {
        Ok(body) => format!("received {} bytes", body.len()), // <- `=>` here leads into a longer expression
        Err(reason) => format!("fetch failed: {reason}"),
    }
}

println!("{}", describe_fetch(Ok("hello".to_string())));
```

Whether the arm's expression is a bare literal or a
multi-step `format!` call, `=>` marks the same boundary — the pattern ends,
the value to produce begins — which is why arms of very different
complexity can sit in the same `match` without special-casing the
separator.

## Explanation (Embedded)

`=>` means exactly the same thing under `#![no_std]` — pure match-arm
grammar, compiling to whatever comparison or jump the pattern needs, no
`std` dependency. It shows up constantly in interrupt-driven firmware,
where a peripheral's status/flags value is read once and matched to
decide which follow-up action to take — the same one-to-one "pattern
leads to an action" shape as any hosted `match`, just with hardware
states standing in for the enum variants.

## Usage examples (Embedded)

### Dispatching on a UART interrupt flag

```
enum UartEvent {
    RxNotEmpty,
    TxEmpty,
    Overrun,
}

fn handle(event: UartEvent) {
    match event {
        UartEvent::RxNotEmpty => { /* read the received byte */ } // <- `=>` separates the flag from its handler
        UartEvent::TxEmpty => { /* push the next byte to transmit */ }
        UartEvent::Overrun => { /* clear the overrun flag, log the drop */ }
    }
}
```

### Mapping a HAL read result onto a defmt log level

```
match sensor.read() {
    Ok(sample) => defmt::info!("sample: {}", sample), // <- `=>` leads into the success path
    Err(_) => defmt::warn!("sensor read failed"),
}
```

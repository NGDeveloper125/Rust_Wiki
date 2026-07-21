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
by an optional final expression.

A block is itself an expression: it evaluates to its final expression (if
it has no trailing `;`), or to `()` otherwise. Function bodies, `if`/`else`
arms, and loop bodies are all block expressions under the hood, which is
exactly why `if` can produce a value. (A `match` arm's body is any
expression — a block is just one option there.)

`{ }` is reused for a completely different purpose in `Type { field: value, ... }`
— a **struct literal** — where it delimits field initializers rather than
statements. The two uses are distinguished purely by what precedes the
brace (a type path vs. nothing), which is also why `if SomeStruct { .. } { }`
needs disambiguating parentheses in condition position — the parser would
otherwise try to read the struct literal as the `if`'s block.

## Usage examples

### Evaluating a block to a value

```
let y = { // <- `{` opens a block expression
    let x = 1;
    x + 1 // no trailing `;`, so this is the block's value
}; // <- `}` closes it; y is now 2
```

### Creating a new object

A struct literal's `{ }` can build the whole value in one expression,
including computing derived fields inline — no separate mutation step
needed afterward.

```
struct Rectangle { width: f64, height: f64, area: f64 }

fn rectangle(width: f64, height: f64) -> Rectangle {
    Rectangle { width, height, area: width * height } // <- `{ }` builds the whole value at once
}
```

Constructing the fully-formed value in one struct
literal, rather than creating a default/partial value and mutating fields
into place, avoids ever having an inconsistent intermediate state (e.g.
`area` not yet matching `width`/`height`) that some other code could
observe.

### Branching on data (pattern matching)

A `match` arm's body is any expression — wrapping it in `{ }` turns it
into a block expression, which is what lets an arm run several statements
before producing its value, while a short arm skips the braces entirely.

```
enum Status { Ok, Error(u16) }

let status = Status::Error(503);
let description = match status {
    Status::Ok => "ready",
    Status::Error(code) => { // <- `{` opens a multi-statement arm body
        eprintln!("request failed with code {code}");
        "failed"
    } // <- `}` closes it; "failed" is this arm's value
};
```

`match` is itself an expression with a single type, so
every arm — braced block or bare expression — must produce that same
type. That's what lets `match` be assigned directly to a binding, rather
than requiring a separate mutable variable set inside each arm.

## Explanation (Embedded)

`{ }` means exactly the same thing under `#![no_std]` — block-expression
and struct-literal delimiter, resolved purely by the compiler with no
`std` dependency. The one place it appears in a distinctly embedded
position is an interrupt handler: `#[interrupt]`/`#[exception]`
(cortex-m-rt) and RTIC task functions are ordinary Rust functions, so
their body is delimited by the same `{ }` as any other `fn` — the
attribute controls *when and how the runtime calls it*, not the grammar
of what's inside.

## Usage examples (Embedded)

### An interrupt handler's body

```
#![no_std]
#![no_main]

use cortex_m_rt::interrupt;

#[interrupt]
fn TIM2() { // <- `{` opens the handler body; ordinary block-expression grammar
    // acknowledge the timer interrupt, service the peripheral
} // <- `}` closes it
```

### Constructing a peripheral config struct literal

```
let config = SerialConfig { baud_rate: 115_200, parity: Parity::None }; // <- `{ }` builds the config value in one expression
```

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
(`HashMap<K, V>`), and match arms in some macro contexts.

A trailing comma after the last element is allowed (and idiomatic in
multi-line lists — `rustfmt` adds it automatically) in every one of these
positions, such as after the final element of a multi-line `vec![1, 2, 3]`
literal.

The one place a single trailing comma is *required*, not just allowed, is
a one-element tuple: `(x,)` — without the comma, `(x)` is just a
parenthesized expression, not a tuple at all.

## Usage examples

### Separating elements in a tuple literal

```
let point = (1, 2, 3);
//            ^  ^ each `,` separates one tuple element from the next
```

**Restriction:** in a one-element tuple, the trailing comma is
*mandatory* — `(x,)` is a tuple, `(x)` is just `x` in parentheses.

### Working with collections

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

Without the trailing comma, inserting a new element
means editing the *previous* line to add a comma too, which shows up as
a noisy, unrelated change in a diff/code review — `rustfmt` inserts
trailing commas in multi-line lists specifically to keep diffs minimal.

### Creating a new object

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

Consistent trailing commas across struct literals,
function calls, and collection literals mean `rustfmt` never has to
special-case one construct — one rule, applied everywhere reduces the
number of style decisions a codebase has to make.

## Explanation (Embedded)

`,` means exactly the same thing under `#![no_std]` — pure separator
grammar, no `std` dependency. It shows up just as constantly in embedded
code, mostly in two places: fixed-size array/buffer literals (there's no
heap-backed `vec![...]` to reach for instead) and peripheral config
structs, where a driver's setup value is built from several named fields
in one struct literal.

## Usage examples (Embedded)

### Building a fixed-size calibration buffer

```
let calibration: [u16; 4] = [512, 511, 509, 515]; // <- `,` separates each calibration sample
```

### Constructing a peripheral config struct

```
let config = SerialConfig {
    baud_rate: 115_200,
    parity: Parity::None,
    stop_bits: StopBits::One, // <- trailing comma, same rustfmt convention as hosted code
};
```

---
title: "."
kind: punctuation
embedded_support: full
groups: [Basics]
related_concepts: [Structs, "Tuple structs"]
related_syntax: ["( )", "*"]
see_also: ["*", "( )"]
---

## Explanation

`.` has three related but distinct uses, all sharing the same "reach into
what's on my left" shape.

**Field access** reads a named field off a struct: `value.field`. **Method
calls** chain the same token: `value.method(args)`, and a chain of calls
reads left to right, each one operating on the previous call's result
(`text.trim().to_lowercase().len()`). **Tuple indexing** uses `.` followed
by an integer literal — `.0`, `.1`, and so on — to reach a field by
position instead of by name, and it applies identically to a plain tuple
(`pair.0`) and to a [tuple struct](../../concepts/types-data-modeling/tuple-structs.md)
(`point.0`), since a tuple struct's fields are addressed the same
positional way a plain tuple's are.

`.` is lexically overloaded with the decimal point in a floating-point
literal (`3.14`) — the tokenizer disambiguates by what follows: a digit
after `.` continues a float literal, while an identifier or another digit
used as a tuple index follows the field-access/tuple-indexing rule
instead. `.` is also unrelated to `..` and `..=` (range syntax) and to `::`
(path separator, see [`::`](../operators/path-separator.md)) — each is its
own token, not `.` repeated or combined with another character.

Both field access and method calls **auto-deref**: writing `my_box.len()`
or `my_reference.field` follows through as many `&`/`*` layers as needed
to find a matching method or field on the pointee, without the caller
writing `(*my_box).len()` by hand. This mechanism is `Deref`-based and is
covered from the token's own angle on the [`*`](../operators/asterisk.md)
page rather than repeated here.

## Basic usage example

```
struct Point { x: f64, y: f64 }

let p = Point { x: 1.0, y: 2.0 };
println!("{}", p.x);       // <- `.` here is field access

let pair = (10, "ten");
println!("{}", pair.0);    // <- `.` here is tuple indexing, not field access

let text = "  Hello  ";
println!("{}", text.trim().to_lowercase()); // <- `.` chains two method calls left to right
```

## Best practices & deeper information

### Scenario: Creating a new object

A builder's methods each take and return `Self`, so `.` chains them into
one expression that reads as a sequence of configuration steps ending in
a final, fully-built value.

```
struct RequestBuilder {
    url: String,
    timeout_ms: u32,
}

impl RequestBuilder {
    fn new(url: impl Into<String>) -> Self {
        RequestBuilder { url: url.into(), timeout_ms: 5000 }
    }

    fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }
}

let request = RequestBuilder::new("https://api.example.com")
    .timeout_ms(2000); // <- `.` chains onto the value the previous call returned
```

**Why this way:** each builder method returning `Self` is what makes
chaining with `.` possible at all — the
[Rust Design Patterns book](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
covers this shape as the idiomatic way to assemble a many-field value
step by step, without needing a temporary mutable variable at the call
site.

### Scenario: Branching on data (pattern matching)

Reaching for `.0`/`.1` instead of a full destructuring pattern is the
right call when only one field of a tuple or tuple struct is needed —
matching every field just to discard most of them adds ceremony a
positional dot access avoids.

```
struct Rgb(u8, u8, u8);

fn red_channel(color: &Rgb) -> u8 {
    color.0 // <- tuple indexing: pulls one field without matching all three
}

// contrast: matching is worth it once every field is actually used
fn describe(color: &Rgb) -> String {
    match *color {
        Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}
```

**Why this way:** `.0` is the more direct choice whenever the rest of the
tuple genuinely isn't needed at that call site; reach for a `match`/`let`
pattern instead the moment more than one field is being pulled out,
since naming each field there reads more clearly than a run of numbered
dot accesses.

## Embedded Rust Notes

**Full support.** Field access, method calls, and tuple indexing are
core-language grammar with no `std` dependency — used identically to
address peripheral register fields and driver state on an embedded
target.

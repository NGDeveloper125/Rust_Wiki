---
title: "'a (named lifetime)"
kind: lifetime
embedded_support: full
groups: ["Ownership & Borrowing"]
related_concepts: ["Lifetimes", "Lifetime elision"]
related_syntax: ["'static", "&"]
see_also: ["'static"]
---

## Explanation

A named lifetime is an apostrophe followed by an identifier — `'a`, `'b`,
`'de`, `'buf` — conventionally a short lowercase name, though any identifier
is legal. It appears in two places: attached to a reference **type**,
`&'a T`, to name the lifetime that reference's validity is tied to; and
inside the angle-bracket generic parameter list of a function, struct,
`impl` block, or trait, `fn parse<'a>(input: &'a str) -> &'a str`, where it
is *declared* before being used anywhere else in the item. A lifetime used
inside a signature or type must first appear in some enclosing generic
parameter list — the same rule that requires a generic type parameter `T`
to be declared before it's used as a bound.

The same `'ident` sigil has a second, entirely unrelated meaning as a
**loop label**: `'outer: loop { ... break 'outer; }` names a `loop`,
`while`, or `for` so that `break`/`continue` can target it specifically from
inside a nested loop, rather than only affecting the innermost one. Nothing
connects the two uses beyond sharing a sigil — a loop label can never be
used where a lifetime is expected and vice versa. Rust's grammar tells them
apart purely by position: an `'ident` immediately followed by a colon and
then `loop`/`while`/`for` is a label; an `'ident` appearing in a type
position or a generic parameter list is a lifetime. (This is the same kind
of position-based disambiguation `&` uses to separate borrow from bitwise
AND — see [`&`](../operators/ampersand.md).)

Within a generic parameter list or `where` clause, `'a: 'b` (read "`'a`
outlives `'b`") declares a constraint *between* two already-introduced
lifetime parameters rather than introducing a new one: whatever concrete
lifetime `'a` ends up being, it is guaranteed to last at least as long as
`'b`. It's written exactly like a trait bound (`T: Trait`), just between two
lifetimes instead of a type and a trait, and shows up whenever a function or
struct needs to hand back a reference borrowed from the longer-lived of two
inputs — see the Scenario below for a worked example.

Since the 2021 edition, a reserved keyword can be used as an ordinary
identifier via the `r#` prefix (`r#move`, `r#async`, …), and that escape
hatch extends to lifetime names too: `'r#move` names a lifetime literally
spelled `move` in a context where the bare word would collide with the
keyword. This is genuinely obscure — it exists mostly as a consistency
guarantee for macro-generated code that constructs identifiers
programmatically, not something to reach for by hand; essentially all real
code sticks to short conventional names like `'a`, `'b`, or a descriptive
`'de` (serde's deserializer lifetime).

## Usage examples

### Declaring a lifetime parameter on a struct

```
struct Excerpt<'a> {
    text: &'a str, // <- `'a` here is the same lifetime declared on the struct
}

let novel = String::from("Call me Ishmael. Some years ago...");
let excerpt = Excerpt { text: &novel[0..17] };
println!("{}", excerpt.text);
```

### Sharing data with multiple references

A type that stores a borrowed slice, rather than owning its data, has to
name a lifetime parameter so the compiler can tie the struct's own validity
to whatever it was built from.

```
struct LogViewer<'a> {
    // <- `'a` declared here, in the struct's generic parameter list
    entries: &'a [String],
}

impl<'a> LogViewer<'a> {
    fn latest(&self) -> Option<&'a str> {
        self.entries.last().map(String::as_str)
    }
}

let history = vec!["startup ok".to_string(), "connected".to_string()];
let viewer = LogViewer { entries: &history }; // <- the borrow's lifetime becomes `'a` for this `viewer`
println!("{:?}", viewer.latest());
```

A struct holding a reference must name a lifetime
parameter tying the struct's own validity to the data it borrows from, so
the compiler can reject any attempt to use `viewer` after `history` goes
away — the
[Book's lifetime chapter](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
covers this as the reason reference-holding structs need `'a` at all.

### Writing generic code

Choosing between a possibly-present override and an always-present
fallback needs an explicit outlives bound so the compiler knows the
fallback's longer-lived reference can stand in wherever the shorter-lived
one is expected.

```
fn resolve_setting<'p, 'f: 'p>(
    // <- `'f: 'p` declares `'f` outlives `'p`, so a `&'f str` can be returned as `&'p str`
    preferred: Option<&'p str>,
    fallback: &'f str,
) -> &'p str {
    preferred.unwrap_or(fallback)
}

let global_default = String::from("utf-8");
let result;
{
    let session_override = String::from("utf-16");
    result = resolve_setting(Some(&session_override), &global_default);
    println!("{result}");
}
```

Without the `'f: 'p` bound, the compiler has no reason to
believe a `&'f str` is valid anywhere a `&'p str` is expected — the
[Rust Reference's lifetime-bounds section](https://doc.rust-lang.org/reference/trait-bounds.html#lifetime-bounds)
documents `'a: 'b` as exactly this kind of outlives constraint between two
declared lifetime parameters.

### Working with collections

Searching a two-dimensional collection for a target value needs to break
out of both the outer and inner loop at once — a loop label, written with
the same `'ident` sigil as a lifetime but meaning something wholly
different, is the direct way to do it.

```
let bins: Vec<Vec<&str>> = vec![
    vec!["screws", "bolts"],
    vec!["nails", "washers"],
];

let mut found = None;

'outer: for (row, shelf) in bins.iter().enumerate() {
    // <- `'outer` here is a loop label, not a lifetime — same sigil, unrelated meaning
    for (col, item) in shelf.iter().enumerate() {
        if *item == "washers" {
            found = Some((row, col));
            break 'outer; // <- exits both loops at once, not just the inner one
        }
    }
}

println!("{found:?}");
```

An unlabeled `break` only ever exits the innermost loop —
the [Rust Reference's labelled block/loop section](https://doc.rust-lang.org/reference/expressions/loop-expr.html#labelled-block-expressions)
documents loop labels as the standard way to target an outer loop
explicitly instead of restructuring the search with a found-flag and extra
checks.

## Embedded Rust Notes

**Full support.** Lifetimes and loop labels are both purely compile-time,
core-language constructs erased before codegen — identical behavior in
`#![no_std]`, with no allocator or runtime footprint either way.

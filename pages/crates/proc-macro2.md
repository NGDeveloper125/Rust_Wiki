---
title: "proc-macro2"
version: "1.0.107"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-24"
summary: "A `TokenStream` that works outside a procedural macro. Mirrors the compiler's `proc_macro` API so macro code can be unit tested, called from ordinary functions, and reused by build scripts."
categories: ["macros", "procedural-macros", "testing"]
repository: "https://github.com/dtolnay/proc-macro2"
---

## Overview

The compiler gives procedural macros a `proc_macro::TokenStream`, and that type
has an awkward property: **it only exists during macro expansion.** Name it in
an ordinary function and the program won't link; call it from a `#[test]` and
you get a panic about being outside a procedural macro. So the natural way to
write a macro — a small function you can test, called from a thin entry point —
isn't available.

`proc-macro2` is a copy of that API which works anywhere. It mirrors the
compiler's types (`TokenStream`, `TokenTree`, `Ident`, `Literal`, `Punct`,
`Group`, `Span`), and converts to and from the real ones at the boundary:

```
use proc_macro2::TokenStream;

// An ordinary function, testable like any other.
fn double(input: TokenStream) -> TokenStream {
    let mut out = input.clone();
    out.extend(input);
    out
}

let tokens: TokenStream = "a b".parse().unwrap();
assert_eq!(double(tokens).to_string(), "a b a b");
```

Inside a real macro you convert once on the way in and once on the way out, and
everything between is testable. That is the whole point, and it is why
[`syn`](syn.md) and [`quote`](quote.md) are both built on this type rather than
on the compiler's — depending on `proc_macro` directly would make them unusable
outside a macro, and untestable themselves.

**How it actually works is worth knowing, because it explains the caveats.** At
runtime the crate picks one of two implementations: when it *is* running inside
a procedural macro it delegates to the compiler, so spans are real and errors
point where they should; otherwise it falls back to its own lexer, where spans
carry no source location. Nothing in the type signatures changes, so the same
code works in both, but a `Span` obtained in a unit test is not the span you get
in production.

Two consequences follow. Line and column information needs the `span-locations`
feature, and even then only in the fallback — inside a macro the compiler owns
that. And `Span::source_text` returns `None` in the fallback, so a macro that
formats error messages from source text behaves differently under test than in
use.

The trio divides up as: **proc-macro2** the tokens, **`syn`** tokens into a
syntax tree, **`quote`** a tree back into tokens. Most macros never name this
crate beyond the entry point's `.into()` calls — it earns its download rank as
the foundation the other two stand on. Reach for its types directly when you are
writing that entry point, building tokens by hand, or working with tokens
outside a macro entirely: a build script, a code generator, a linter.

It is one dependency (`unicode-ident`), requires Rust 1.71, and is not `no_std`.

## When to use it

### Use case: A macro entry point you can test

The compiler's `TokenStream` at the edges, `proc_macro2`'s everywhere else. The
real logic becomes a normal function.

```
use proc_macro2::TokenStream;
use quote::quote;

// All the work, in a function tests can call.
fn expand(input: TokenStream) -> TokenStream {
    quote! {
        fn generated() -> &'static str {
            stringify!(#input)
        }
    }
}

// The entry point converts at the boundary and does nothing else. In a
// proc-macro crate this carries #[proc_macro] and takes proc_macro::TokenStream:
//     let out = expand(input.into());
//     out.into()

let out = expand("hello".parse().unwrap());
assert!(out.to_string().contains("fn generated"));
```

**Why it fits:** `expand` is testable, debuggable and callable from a benchmark.
Written against `proc_macro::TokenStream` the same function could only be
exercised by compiling a crate that uses the macro and checking whether it built.

### Use case: Inspecting tokens without parsing them

Not every macro needs a syntax tree. Counting, filtering or checking the shape
of a token stream is often simpler directly.

```
use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// How deeply nested are the braces in this stream?
fn max_depth(tokens: TokenStream) -> usize {
    fn walk(tokens: TokenStream, depth: usize) -> usize {
        tokens
            .into_iter()
            .map(|tree| match tree {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                    walk(g.stream(), depth + 1)
                }
                TokenTree::Group(g) => walk(g.stream(), depth),
                _ => depth,
            })
            .max()
            .unwrap_or(depth)
    }
    walk(tokens, 0)
}

assert_eq!(max_depth("fn a() { if x { } }".parse().unwrap()), 2);
assert_eq!(max_depth("let a = 1;".parse().unwrap()), 0);
```

**Why it fits:** a `TokenStream` is already a tree of balanced groups, so
structural questions can be answered by walking it. Pulling in `syn`'s `full`
feature to parse a whole function, just to count braces, would cost far more
build time than it saves.

### Use case: Generating code outside a macro

A build script emitting a `.rs` file has no compiler `TokenStream` available at
all — but `syn` and `quote` still work, because they are built on this crate.

```
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// The kind of thing a build.rs writes to OUT_DIR.
fn generate_constants(names: &[&str]) -> TokenStream {
    let idents: Vec<Ident> = names
        .iter()
        .map(|n| Ident::new(&n.to_uppercase(), Span::call_site()))
        .collect();
    let values = 0u32..names.len() as u32;

    quote! {
        #( pub const #idents: u32 = #values; )*
    }
}

let code = generate_constants(&["alpha", "beta"]).to_string();
assert!(code.contains("pub const ALPHA : u32 = 0u32 ;"));
assert!(code.contains("pub const BETA : u32 = 1u32 ;"));
```

**Why it fits:** the same toolkit that writes macro output writes generated
files, and the result is tokens rather than a string — so it cannot be
malformed by a missing space, and `prettyplease` can format it. Turning off the
default `proc-macro` feature drops the compiler bridge entirely for this use.

## API map

The types mirror `proc_macro`'s, so anything you learn here transfers. Entries
below cover constructing and taking apart token streams, plus the span handling
that decides where errors land.

### Token streams

#### `TokenStream`

The central type: a sequence of `TokenTree`s. Build it by parsing, by
collecting, or with `quote!`.

```
use proc_macro2::TokenStream;

let parsed: TokenStream = "let x = 1;".parse().unwrap();
assert_eq!(parsed.clone().into_iter().count(), 5);

let empty = TokenStream::new();
assert!(empty.is_empty());

// Parsing rejects unbalanced delimiters, but not invalid Rust.
assert!("fn f( {".parse::<TokenStream>().is_err());
assert!("let let let".parse::<TokenStream>().is_ok()); // <- tokens fine, syntax not
```

**When to use it:** as the currency of every macro. Note what parsing does and
doesn't check — delimiters must balance, but the result needn't be valid Rust.
That is `syn`'s job, and the split is deliberate: a macro with its own syntax
still needs the tokens.

#### Iterating a stream

`IntoIterator` yields `TokenTree`s, so a stream can be walked, filtered and
rebuilt with ordinary iterator code.

```
use proc_macro2::{TokenStream, TokenTree};

let tokens: TokenStream = "a + b".parse().unwrap();

let idents: Vec<String> = tokens
    .into_iter()
    .filter_map(|tree| match tree {
        TokenTree::Ident(i) => Some(i.to_string()),
        _ => None,
    })
    .collect();

assert_eq!(idents, ["a", "b"]);
```

**When to use it:** for structural work that doesn't need a syntax tree.
Iteration is shallow — a `Group` arrives as one item, and you recurse into
`g.stream()` yourself, which is what makes nesting explicit rather than
accidental.

#### `extend` and `FromIterator`

Streams concatenate, so pieces can be built separately and joined.

```
use proc_macro2::TokenStream;

let mut tokens: TokenStream = "let x".parse().unwrap();
tokens.extend("= 1 ;".parse::<TokenStream>());

assert_eq!(tokens.to_string(), "let x = 1 ;");

// Or collect trees straight into a stream.
let collected: TokenStream = tokens.into_iter().take(2).collect();
assert_eq!(collected.to_string(), "let x");
```

**When to use it:** assembling output across branches or a loop. In practice
`quote!`'s interpolation covers most of it; reach for `extend` when the
structure is decided by control flow that `#(...)` can't express.

### Token trees

#### `TokenTree`

The four things a stream can contain: `Group`, `Ident`, `Punct` and `Literal`.

```
use proc_macro2::{TokenStream, TokenTree};

let tokens: TokenStream = "f(1)".parse().unwrap();
let kinds: Vec<&str> = tokens
    .into_iter()
    .map(|tree| match tree {
        TokenTree::Group(_) => "group",
        TokenTree::Ident(_) => "ident",
        TokenTree::Punct(_) => "punct",
        TokenTree::Literal(_) => "literal",
    })
    .collect();

assert_eq!(kinds, ["ident", "group"]); // <- the `(1)` is one group
```

**When to use it:** whenever you match on tokens. The example shows the thing
that surprises people first: `f(1)` is *two* trees, because everything inside
the parentheses is nested in the group rather than sitting alongside it.

#### `Ident`

An identifier, with a span.

```
use proc_macro2::{Ident, Span};

let name = Ident::new("widget", Span::call_site());
assert_eq!(name.to_string(), "widget");

// Raw identifiers let you name something that is otherwise a keyword.
let raw = Ident::new_raw("type", Span::call_site());
assert_eq!(raw.to_string(), "r#type");
```

**When to use it:** constructing names in generated code, though `quote`'s
`format_ident!` is nicer when the name is derived from another. `Ident::new`
panics on an invalid identifier — including on a keyword, which is what
`new_raw` is for.

#### `Punct` and `Spacing`

A single punctuation character, plus whether the next token is joined to it.

```
use proc_macro2::{Punct, Spacing, TokenStream, TokenTree};

// `->` is two Puncts: the first Joint, the second Alone.
let tokens: TokenStream = "->".parse().unwrap();
let spacings: Vec<Spacing> = tokens
    .into_iter()
    .filter_map(|t| match t {
        TokenTree::Punct(p) => Some(p.spacing()),
        _ => None,
    })
    .collect();

assert_eq!(spacings, [Spacing::Joint, Spacing::Alone]);

let arrow = Punct::new('-', Spacing::Joint);
assert_eq!(arrow.as_char(), '-');
```

**When to use it:** building or recognising multi-character operators. Spacing is
the only thing distinguishing `->` from `-` followed by `>`, so getting it wrong
produces tokens that print identically and parse differently — a genuinely
confusing bug.

#### `Literal`

A literal value: number, string, char or byte string.

```
use proc_macro2::Literal;

assert_eq!(Literal::u32_suffixed(7).to_string(), "7u32");
assert_eq!(Literal::u32_unsuffixed(7).to_string(), "7");
assert_eq!(Literal::string("hi").to_string(), "\"hi\"");
assert_eq!(Literal::character('x').to_string(), "'x'");
```

**When to use it:** emitting constants. Suffixed pins the type; unsuffixed lets
inference decide, which is usually what you want inside generated code so the
value adapts to its context. `Literal::string` handles escaping, so building a
string literal by concatenating quotes is never necessary.

#### `Group` and `Delimiter`

A delimited sub-stream: `()`, `[]`, `{}`, or none.

```
use proc_macro2::{Delimiter, Group, TokenStream};

let inner: TokenStream = "1 + 2".parse().unwrap();
let group = Group::new(Delimiter::Parenthesis, inner);

assert_eq!(group.to_string(), "(1 + 2)");
assert_eq!(group.delimiter(), Delimiter::Parenthesis);
assert_eq!(group.stream().into_iter().count(), 3);
```

**When to use it:** constructing bracketed output by hand, and reaching into a
nested stream when walking one. `Delimiter::None` is the invisible grouping the
compiler uses to keep an interpolated expression's precedence intact — you will
see it in input, and rarely need to create it.

### Spans

#### `Span::call_site`

The default span: names resolve at the macro's call site, and errors are
reported there.

```
use proc_macro2::{Ident, Span};

let name = Ident::new("generated", Span::call_site());
assert_eq!(name.to_string(), "generated");
```

**When to use it:** for tokens you invented, which is most of them. It is what
`quote!` uses. The alternative, `Span::mixed_site`, gives macro-hygienic
resolution for local variables — worth knowing when a generated `let` might
collide with one of the user's.

#### `Span::source_text`

The original source behind a span, when there is one.

```
use proc_macro2::TokenStream;

let tokens: TokenStream = "alpha".parse().unwrap();
let first = tokens.into_iter().next().unwrap();

// None here: the fallback lexer has no source file behind it.
assert!(first.span().source_text().is_none());
```

**When to use it:** quoting the user's own code back at them in an error
message. Treat `None` as normal rather than exceptional — that is what you get
in the fallback, so a macro must still produce a sensible message without it.

#### `set_span`

Retags a token, which is how you move blame onto a different piece of code.

```
use proc_macro2::{Ident, Span, TokenStream, TokenTree};

let tokens: TokenStream = "target".parse().unwrap();
let target_span = tokens.into_iter().next().unwrap().span();

let mut generated = TokenTree::Ident(Ident::new("check", Span::call_site()));
generated.set_span(target_span); // <- errors now point at `target`

assert_eq!(generated.to_string(), "check");
```

**When to use it:** when tokens are built somewhere that doesn't know the right
span and retagged later. For code written inline, `quote_spanned!` says the same
thing more clearly, and should be the first choice.

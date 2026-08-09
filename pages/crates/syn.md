---
title: "syn"
version: "3.0.3"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-08"
summary: "Parses a stream of Rust tokens into a syntax tree. The parsing half of every procedural macro — `DeriveInput` for derives, the `Parse` trait for custom syntax, and spanned errors that point at the user's code."
categories: ["macros", "procedural-macros", "parsing"]
repository: "https://github.com/dtolnay/syn"
---

## Overview

A [procedural macro](../concepts/macros-metaprogramming/procedural-macros.md)
receives a `TokenStream` — a flat sequence of identifiers, literals, punctuation
and bracketed groups — and must give one back. Nothing in that type knows what a
struct is. `syn` is the missing half: it parses those tokens into a syntax tree
of Rust, so `#[derive(MyTrait)]` can ask "what are this struct's fields?" instead
of walking punctuation by hand.

It is the middle crate of the procedural-macro trio, and the three divide up
cleanly:

- **`proc-macro2`** — a `TokenStream` you can use outside a macro, so parsing is
  testable in a normal unit test.
- **`syn`** — tokens in, syntax tree out.
- **[`quote`](https://crates.io/crates/quote)** — syntax tree in, tokens out.

Most macros use all three, and syn's `printing` feature (on by default) pulls in
`quote` so that `parse_quote!` works.

**The cost is compile time, and it is the crate's defining trade-off.** syn's
syntax tree covers nearly all of Rust, and any crate that derives anything drags
it into the build graph — which is why it sits near the top of the download
charts without most people ever writing `use syn`. The mitigation is feature
gating, which syn takes further than almost any crate:

| Feature | Default | What it buys |
| --- | --- | --- |
| `derive` | yes | `DeriveInput` and the types a `#[derive]` needs |
| `parsing` | yes | Tokens → tree |
| `printing` | yes | Tree → tokens, via `quote` |
| `clone-impls` | yes | `Clone` on the tree types |
| `proc-macro` | yes | Bridging to the real `proc_macro::TokenStream` |
| `full` | **no** | Items, expressions, statements — everything beyond a derive |
| `extra-traits` | no | `Debug`, `Eq`, `Hash` on tree types — useful while developing |
| `visit`, `visit-mut`, `fold` | no | Generated traversal traits |

A derive macro needs only the defaults. Reach for `full` only when you parse
whole functions or expressions, and turn `default-features = false` on if you're
using syn to parse a small custom syntax rather than Rust itself.

If your macro is genuinely trivial, syn may be more than you need:
[`venial`](https://crates.io/crates/venial) describes itself as "a very small
syn" and parses only the declarations a derive sees, for a much lighter build;
and a macro that only wraps its input can work on `proc-macro2` alone. But for
anything that inspects Rust beyond that, syn is the standard
answer, it is maintained by the author of `quote`, `thiserror` and `serde`'s
derive, and it requires Rust 1.71.

## When to use it

### Use case: A derive macro reading a struct's fields

The archetypal job. `DeriveInput` is any of the three things a `#[derive]` can be
applied to, and matching on its `Data` gets you to the fields.

```
use syn::{parse_str, Data, DeriveInput, Fields};

let input: DeriveInput = parse_str(
    "struct Config { host: String, port: u16 }",
).unwrap();

let mut names = Vec::new();
if let Data::Struct(data) = &input.data {
    if let Fields::Named(fields) = &data.fields {
        for field in &fields.named {
            // Named fields always have an ident.
            names.push(field.ident.as_ref().unwrap().to_string());
        }
    }
}

assert_eq!(input.ident.to_string(), "Config");
assert_eq!(names, ["host", "port"]);
```

**Why it fits:** the tree gives you the struct's name, its generics and its
fields as typed data. Doing this on a raw `TokenStream` means reimplementing
Rust's grammar, including the cases you forgot — tuple structs, generics with
defaults, `where` clauses.

### Use case: Reading arguments out of a helper attribute

Once a derive has options — `#[column(name = "id", skip)]` — you need to parse
the attribute's contents. `parse_nested_meta` walks the comma-separated list and
hands you each entry, so you never touch a token.

```
use syn::{parse_str, Attribute, DeriveInput, LitStr};

let input: DeriveInput = parse_str(
    r#"#[column(name = "id", skip)] struct Row;"#,
).unwrap();

let mut name = None;
let mut skip = false;

for attr in &input.attrs {
    if !attr.path().is_ident("column") {
        continue; // <- not ours; leave other macros' attributes alone
    }
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
            let value: LitStr = meta.value()?.parse()?;
            name = Some(value.value());
            Ok(())
        } else if meta.path.is_ident("skip") {
            skip = true;
            Ok(())
        } else {
            Err(meta.error("unrecognised column option"))
        }
    })
    .unwrap();
}

assert_eq!(name.as_deref(), Some("id"));
assert!(skip);
let _: fn(&Attribute) = |_| {};
```

**Why it fits:** the unknown-option branch produces a real compile error at the
offending token rather than a silently ignored typo — which is what makes the
difference between a macro that's pleasant to use and one that isn't.

### Use case: A function-like macro with its own syntax

`sql!(SELECT name, id FROM users)` isn't Rust, and no amount of syntax-tree
types will parse it. Implementing `Parse` lets you define the grammar yourself
while still borrowing syn's tokenising, lookahead and error reporting.

```
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_str, Ident, Token};

// `SELECT a, b FROM table`
struct Select {
    columns: Punctuated<Ident, Token![,]>,
    table: Ident,
}

impl Parse for Select {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let select: Ident = input.parse()?;
        if select != "SELECT" {
            return Err(syn::Error::new(select.span(), "expected SELECT"));
        }
        let mut columns = Punctuated::new();
        loop {
            columns.push_value(input.parse()?);
            if input.peek(Token![,]) {
                columns.push_punct(input.parse()?);
            } else {
                break;
            }
        }
        let from: Ident = input.parse()?;
        if from != "FROM" {
            return Err(syn::Error::new(from.span(), "expected FROM"));
        }
        Ok(Select { columns, table: input.parse()? })
    }
}

let query: Select = parse_str("SELECT name, id FROM users").unwrap();
assert_eq!(query.columns.len(), 2);
assert_eq!(query.table.to_string(), "users");

// Wrong syntax fails with a span, not a panic.
assert!(parse_str::<Select>("PICK name FROM users").is_err());
```

**Why it fits:** you get a parser combinator library that already understands
Rust tokens, balanced delimiters and spans, without inheriting Rust's grammar
where you don't want it.

## API map

Entries below assume the default features; anything needing `full` says so. The
examples parse from strings via `parse_str` because that runs in a normal test —
inside a real macro the same types arrive through `parse_macro_input!`.

### Getting tokens into the tree

#### `parse_macro_input!`

The entry point inside a `#[proc_macro]` function. It parses, and on failure
returns the compile error from your macro instead of panicking.

```
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

// In a crate with `proc-macro = true`, this carries #[proc_macro_derive(MyTrait)].
pub fn my_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let _name = input.ident; // <- a syntax tree from here on
    TokenStream::new()
}
```

**When to use it:** in every macro entry point, and nowhere else — it expands to
an early `return`, so it only works in a function returning `TokenStream`. Use
`syn::parse2` when you need the `Result`.

#### `syn::parse2`

Parses a `proc_macro2::TokenStream`, returning `Result`. This is what makes a
macro testable: `proc_macro2` works outside a compiler invocation, so the parse
step can be unit tested.

```
use syn::{parse2, DeriveInput};

let tokens: proc_macro2::TokenStream = "struct S { a: u8 }".parse().unwrap();
let parsed: DeriveInput = parse2(tokens).unwrap();
assert_eq!(parsed.ident.to_string(), "S");
```

**When to use it:** the workhorse behind `parse_macro_input!`, and the form to
call directly in tests and helper functions. Prefer it to `syn::parse`, which
takes the real `proc_macro::TokenStream` and therefore only runs inside a macro.

#### `syn::parse_str`

Parses any syntax-tree node from a `&str`.

```
use syn::{parse_str, Type};

let ty: Type = parse_str("Vec<Option<T>>").unwrap();
assert!(matches!(ty, Type::Path(_)));
```

**When to use it:** tests, tooling and code generators that read Rust from
somewhere other than a macro invocation. It loses span information relative to
the user's source, so errors point into the string, not their file.

#### `syn::parse_file`

Parses a whole `.rs` file, shebang and inner attributes included. Requires the
`full` feature.

```
use syn::parse_file;

let file = parse_file("//! docs\nfn main() {}\n").unwrap();
assert_eq!(file.items.len(), 1);
assert_eq!(file.attrs.len(), 1); // <- the inner //! doc comment
```

**When to use it:** linters, codegen and refactoring tools that work over source
files. Not for macros — a macro never sees a whole file.

### The derive input

#### `DeriveInput`

The parsed form of whatever a derive was applied to: name, visibility,
generics, attributes and `data`.

```
use syn::{parse_str, DeriveInput};

let input: DeriveInput = parse_str("pub struct Point<T> { x: T }").unwrap();
assert_eq!(input.ident.to_string(), "Point");
assert_eq!(input.generics.params.len(), 1);
assert!(input.attrs.is_empty());
```

**When to use it:** as the parse target of every derive macro. For an attribute
macro use `ItemFn`, `ItemStruct` or `Item` instead — those need `full`.

#### `Data` and `Fields`

`Data` splits struct from enum from union; `Fields` splits named from unnamed
from unit. Both are the matches every derive macro starts with.

```
use syn::{parse_str, Data, DeriveInput, Fields};

fn field_count(src: &str) -> usize {
    let input: DeriveInput = parse_str(src).unwrap();
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => f.named.len(),
            Fields::Unnamed(f) => f.unnamed.len(),
            Fields::Unit => 0,
        },
        Data::Enum(data) => data.variants.len(),
        Data::Union(data) => data.fields.named.len(),
    }
}

assert_eq!(field_count("struct A { x: u8, y: u8 }"), 2);
assert_eq!(field_count("struct B(u8);"), 1);
assert_eq!(field_count("struct C;"), 0);
assert_eq!(field_count("enum D { X, Y, Z }"), 3);
```

**When to use it:** immediately after parsing, and handle all the arms. A derive
that only matches `Fields::Named` breaks on tuple structs — usually with a
confusing error, because the user's mistake was applying a perfectly reasonable
derive.

#### `Field`

One field: its `ident` (`None` for tuple fields), `ty`, `vis` and `attrs`.

```
use syn::{parse_str, Data, DeriveInput, Fields};

let input: DeriveInput = parse_str("struct S { pub name: String }").unwrap();
let Data::Struct(data) = &input.data else { unreachable!() };
let Fields::Named(fields) = &data.fields else { unreachable!() };
let field = fields.named.first().unwrap();

assert_eq!(field.ident.as_ref().unwrap().to_string(), "name");
assert!(matches!(field.vis, syn::Visibility::Public(_)));
```

**When to use it:** whenever you generate per-field code. `field.attrs` is where
helper attributes like `#[serde(skip)]` live, so it's the pair to
`parse_nested_meta`.

### Attributes

#### `Attribute::path`

The attribute's path — what you match on to decide whether it is yours.

```
use syn::{parse_str, DeriveInput};

let input: DeriveInput = parse_str("#[doc = \"hi\"] #[mine] struct S;").unwrap();
let paths: Vec<String> = input
    .attrs
    .iter()
    .map(|a| a.path().get_ident().unwrap().to_string())
    .collect();
assert_eq!(paths, ["doc", "mine"]);
```

**When to use it:** first thing in any attribute loop, via
`attr.path().is_ident("yours")`. Skipping this and parsing every attribute makes
your macro fail on `#[doc]`, `#[cfg]` and every other macro's helpers.

#### `Attribute::parse_nested_meta`

Parses `#[name(a = "x", b, c(1))]` — the conventional shape — calling your
closure once per entry.

```
use syn::{parse_str, DeriveInput, LitStr};

let input: DeriveInput = parse_str(r#"#[opt(rename = "id")] struct S;"#).unwrap();
let mut rename = None;
input.attrs[0]
    .parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            rename = Some(meta.value()?.parse::<LitStr>()?.value());
        }
        Ok(())
    })
    .unwrap();
assert_eq!(rename.as_deref(), Some("id"));
```

**When to use it:** for any attribute following the `#[attr(key = value, flag)]`
convention, which is nearly all of them. Return `meta.error(..)` for unknown
keys so typos are caught at the right span.

#### `Attribute::parse_args`

Parses the attribute's parenthesised contents as one syntax-tree node.

```
use syn::{parse_str, DeriveInput, Expr};

let input: DeriveInput = parse_str("#[guard(x > 3)] struct S;").unwrap();
let condition: Expr = input.attrs[0].parse_args().unwrap();
assert!(matches!(condition, Expr::Binary(_)));
```

**When to use it:** when the argument is a single thing — an expression, a type,
a path — rather than a comma-separated option list. `parse_args_with` takes a
parser function for anything more involved, such as
`Punctuated::<Type, Token![,]>::parse_terminated`.

### Writing a parser

#### The `Parse` trait

`fn(ParseStream) -> Result<Self>`. Implementing it makes a type usable with
`parse_str`, `parse2`, `input.parse()` and `parse_macro_input!`.

```
use syn::parse::{Parse, ParseStream};
use syn::{parse_str, Ident, Token};

struct Rename {
    from: Ident,
    to: Ident,
}

impl Parse for Rename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let from = input.parse()?;
        input.parse::<Token![=>]>()?; // <- Token! builds the token type
        let to = input.parse()?;
        Ok(Rename { from, to })
    }
}

let renamed: Rename = parse_str("old => new").unwrap();
assert_eq!(renamed.from.to_string(), "old");
assert_eq!(renamed.to.to_string(), "new");
```

**When to use it:** for every custom syntax your macro accepts. Composing
`Parse` types is how a grammar is built — each one parses its own piece and
delegates the rest.

#### `ParseStream::peek` and `Lookahead1`

`peek` tests the next token without consuming it. `lookahead1` does the same but
remembers everything you tried, so the failure message lists all of it.

```
use syn::parse::{Parse, ParseStream};
use syn::{parse_str, Ident, LitInt, Token};

enum Arg {
    Name(Ident),
    Index(LitInt),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(Arg::Name(input.parse()?))
        } else if lookahead.peek(LitInt) {
            Ok(Arg::Index(input.parse()?))
        } else {
            Err(lookahead.error()) // <- "expected identifier or integer literal"
        }
    }
}

assert!(matches!(parse_str::<Arg>("field").unwrap(), Arg::Name(_)));
assert!(matches!(parse_str::<Arg>("7").unwrap(), Arg::Index(_)));
let _: bool = parse_str::<Arg>("'a'").is_err();
```

**When to use it:** any branch point in a grammar. Prefer `lookahead1` over bare
`peek` when the branch can fail — the generated message names every alternative,
which bare `peek` cannot do.

#### `Punctuated`

A sequence with separators, keeping both so it can be printed back out.
`parse_terminated` accepts an optional trailing separator.

```
use syn::punctuated::Punctuated;
use syn::{parse_str, Ident, Token};
use syn::parse::Parser;

let parser = Punctuated::<Ident, Token![,]>::parse_terminated;
let idents = parser.parse_str("a, b, c,").unwrap(); // <- trailing comma is fine

assert_eq!(idents.len(), 3);
assert_eq!(idents.first().unwrap().to_string(), "a");
```

**When to use it:** every comma-separated list — arguments, fields, generic
parameters. `parse_separated_nonempty` is the variant that rejects a trailing
separator, for grammars where it would be wrong.

#### `braced!`, `bracketed!` and `parenthesized!`

Consume a balanced group and give you a `ParseStream` over its contents.

```
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parse_str, Ident, Token};

struct Block {
    names: Punctuated<Ident, Token![,]>,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input); // <- content is a ParseStream for the { .. }
        Ok(Block { names: content.parse_terminated(Ident::parse, Token![,])? })
    }
}

let block: Block = parse_str("{ a, b }").unwrap();
assert_eq!(block.names.len(), 2);
```

**When to use it:** any nested syntax. Because the token stream is already a
tree of balanced groups, these can't fail to match a delimiter — an unclosed
brace was rejected before syn ever saw it.

#### `custom_keyword!`

Defines a keyword that Rust doesn't have, complete with a peekable token type.

```
use syn::parse::{Parse, ParseStream};
use syn::{custom_keyword, parse_str, Ident};

custom_keyword!(select);

struct Query {
    table: Ident,
}

impl Parse for Query {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<select>()?; // <- matches the identifier `select`
        Ok(Query { table: input.parse()? })
    }
}

let query: Query = parse_str("select users").unwrap();
assert_eq!(query.table.to_string(), "users");
assert!(parse_str::<Query>("delete users").is_err());
```

**When to use it:** for DSL keywords. Better than comparing an `Ident` to a
string, because the generated type works with `peek` and `lookahead1` and so
produces the right error message.

### Generating code back out

#### `Generics::split_for_impl`

Splits a type's generics into the three fragments an `impl` block needs, so
generic types don't break your derive.

```
use quote::quote;
use syn::{parse_str, DeriveInput};

let input: DeriveInput = parse_str("struct S<T: Clone>(T) where T: Send;").unwrap();
let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
let name = &input.ident;

let expanded = quote! {
    impl #impl_generics Trait for #name #ty_generics #where_clause {}
};
assert!(expanded.to_string().contains("where"));
```

**When to use it:** in every derive that emits an `impl`. Writing
`impl<#generics>` by hand puts bounds and defaults in the wrong places and drops
the `where` clause — this is the single most common bug in a first derive macro.

#### `parse_quote!`

Builds a syntax-tree node from quasi-quoted tokens — `quote!` and a parse in one
step. Requires the `printing` feature, which is on by default.

```
use syn::{parse_quote, Type, WherePredicate};

let field: Type = parse_quote!(Vec<String>);
assert!(matches!(field, Type::Path(_)));

// Interpolation works as in quote!.
let bound: WherePredicate = parse_quote!(#field: Send);
let _ = bound;
```

**When to use it:** constructing tree nodes to splice into what you're building
— adding a `where` clause, synthesising a type. Far more readable than
assembling the node's struct literal by hand.

### Errors and spans

#### `Error::new_spanned`

Builds an error pointing at an existing piece of syntax, so the compiler
underlines exactly that token.

```
use syn::{parse_str, Data, DeriveInput, Error};

let input: DeriveInput = parse_str("enum E { A }").unwrap();
let result = match &input.data {
    Data::Struct(_) => Ok(()),
    _ => Err(Error::new_spanned(&input.ident, "this derive only supports structs")),
};

let err = result.unwrap_err();
assert_eq!(err.to_string(), "this derive only supports structs");
```

**When to use it:** every rejection in a macro. `Error::new(span, msg)` is the
form when you hold a `Span` rather than a node; both beat `panic!`, which
reports at the macro's own call site with no useful location.

#### `Error::to_compile_error`

Turns the error into tokens that expand to `compile_error!`, so the macro can
return it.

```
use syn::Error;
use proc_macro2::Span;

let err = Error::new(Span::call_site(), "unsupported");
let tokens = err.to_compile_error();
assert!(tokens.to_string().contains("compile_error"));
```

**When to use it:** at the top level of a macro, converting `Result` into the
`TokenStream` you must return. `into_compile_error` is the by-value form and is
usually what you want at the end of a chain.

#### `Error::combine`

Merges another error in, so one compilation reports every problem instead of
one at a time.

```
use syn::Error;
use proc_macro2::Span;

let mut err = Error::new(Span::call_site(), "first problem");
err.combine(Error::new(Span::call_site(), "second problem"));

assert_eq!(err.into_iter().count(), 2); // <- both surface at once
```

**When to use it:** when validating a list — every bad field, not just the first.
Users recompile once per error you don't report.

#### `Spanned::span`

Gets the `Span` of any syntax-tree node, covering the whole node rather than its
first token.

```
use syn::spanned::Spanned;
use syn::{parse_str, Expr};

let expr: Expr = parse_str("a + b").unwrap();
let span = expr.span();
let _ = span.source_text(); // <- Some("a + b") when parsed from real source

let err = syn::Error::new(span, "bad expression");
assert_eq!(err.to_string(), "bad expression");
```

**When to use it:** to point an error at a whole construct — an entire type or
expression — rather than where it begins. Import the trait; `span()` isn't an
inherent method.
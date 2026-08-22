---
title: "quote"
version: "1.0.47"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-22"
summary: "Quasi-quoting for procedural macros: write the Rust you want to generate as Rust, and interpolate values into it with `#name`. The output half of every macro, opposite `syn`."
categories: ["macros", "procedural-macros", "codegen"]
repository: "https://github.com/dtolnay/quote"
---

## Overview

A procedural macro has to return a `TokenStream`. Building one by hand means
pushing `Ident`, `Punct` and `Group` values in the right order — the code you
want to emit, written as a description of its own punctuation. `quote!` lets you
write it as Rust instead:

```
use quote::quote;

let name = quote::format_ident!("Widget");

let tokens = quote! {
    impl #name {
        fn new() -> Self {
            #name
        }
    }
};

assert!(tokens.to_string().contains("impl Widget"));
```

`#name` interpolates a value into the output. Everything else is copied through
as tokens.

It is the output half of the procedural-macro trio, and reads most clearly
against its opposite:

- **[`syn`](syn.md)** — tokens in, syntax tree out.
- **`quote`** — syntax tree (or anything implementing `ToTokens`) in, tokens out.
- **`proc-macro2`** — the `TokenStream` type both use, available outside a macro
  so the whole thing is testable in an ordinary unit test.

**What `quote!` is not is a string.** It produces tokens, so `#name` is
interpolated as an *identifier*, not spliced as text — the difference that makes
generated code impossible to break with quoting or whitespace, and lets errors
in the output point at the right place. It also means `quote!` does not check
that what you wrote is valid Rust: it checks the tokens balance, and the
compiler complains later, at the expansion site. A missing semicolon inside
`quote!` compiles fine and fails in your user's crate, which is the main way
this is frustrating to debug. `cargo expand` is how you see what you actually
produced.

The other thing worth internalising early is **spans**, because they decide
where a compile error points. Tokens from `quote!` carry `Span::call_site()`,
which resolves names at the macro's call site and reports errors there.
`quote_spanned!` overrides that, and is how you make a trait-bound failure
underline the user's field instead of your macro.

It is a small, mature dependency by the author of `syn` and `serde`'s derive:
one runtime dependency (`proc-macro2`), Rust 1.71, and no `no_std` support since
it exists to build proc-macro output. Turning off the default `proc-macro`
feature lets it be used outside a macro — for a code generator writing to a
file, say — at the cost of the bridge to the compiler's own `TokenStream`.

## When to use it

### Use case: Emitting an impl from a derive

The archetypal job. `syn` parses the input, `quote!` writes the impl, and the
generics come through `split_for_impl` so generic types don't break.

```
use quote::quote;
use syn::{parse_str, DeriveInput};

let input: DeriveInput = parse_str("struct Point<T> { x: T, y: T }").unwrap();
let name = &input.ident;
let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

let expanded = quote! {
    impl #impl_generics Describe for #name #ty_generics #where_clause {
        fn describe() -> &'static str {
            stringify!(#name)
        }
    }
};

let out = expanded.to_string();
assert!(out.contains("impl < T > Describe for Point < T >"));
```

**Why it fits:** the generated code is legible as code. The alternative —
`tokens.append(Ident::new("impl", span))` and thirty more lines — is the same
thing written in a form nobody can review.

### Use case: Generating one line per field

`#(...)*` repeats a block once per element of an iterable, which is how a derive
walks a struct's fields.

```
use quote::quote;
use syn::{parse_str, Data, DeriveInput, Fields};

let input: DeriveInput = parse_str("struct Config { host: String, port: u16 }").unwrap();
let Data::Struct(data) = &input.data else { unreachable!() };
let Fields::Named(fields) = &data.fields else { unreachable!() };

let names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();

let expanded = quote! {
    fn field_names() -> Vec<&'static str> {
        vec![ #( stringify!(#names) ),* ] // <- comma-separated repetition
    }
};

let out = expanded.to_string();
assert!(out.contains("stringify ! (host)"));
assert!(out.contains("stringify ! (port)"));
```

**Why it fits:** the separator is part of the syntax — `),*` puts a comma
between elements and not after the last one, which is exactly the fiddly bit
when building a list by hand.

### Use case: Making an error point at the user's code

By default a generated trait bound that fails reports at the call site, naming
your macro rather than the field at fault. `quote_spanned!` attaches a field's
span to the tokens that check it.

```
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_str, Data, DeriveInput, Fields};

let input: DeriveInput = parse_str("struct S { a: u8, b: String }").unwrap();
let Data::Struct(data) = &input.data else { unreachable!() };
let Fields::Named(fields) = &data.fields else { unreachable!() };

// One assertion per field, each blamed on that field's own span.
let checks = fields.named.iter().map(|f| {
    let ty = &f.ty;
    quote_spanned! { f.span() =>
        struct _AssertCopy where #ty: Copy;
    }
});

let expanded = quote! { #(#checks)* };
assert!(expanded.to_string().contains("Copy"));
```

**Why it fits:** when `String: Copy` fails, the compiler underlines `b: String`
in the user's struct rather than the `#[derive(...)]` line. That difference is
most of what separates a derive that is pleasant to use from one that is not.

## API map

`quote` is a small crate: two macros that build tokens, one that builds
identifiers, and the traits that decide what may be interpolated. Examples here
use `proc_macro2::TokenStream`, which is what `quote!` produces.

### Building tokens

#### `quote!`

Produces a `proc_macro2::TokenStream` from Rust-shaped input, interpolating
`#var`.

```
use quote::quote;

let ty = quote! { Vec<String> };
let tokens = quote! {
    fn names() -> #ty {
        Vec::new()
    }
};

assert_eq!(tokens.to_string(), "fn names () -> Vec < String > { Vec :: new () }");
```

**When to use it:** everywhere you build macro output. Note the `to_string()`
spacing — tokens are re-printed from the stream, not from your source, so
comparing generated code as text needs care. Assert on `contains`, or parse it
back with `syn`.

#### `quote_spanned!`

`quote!` with an explicit span for every token it produces.

```
use proc_macro2::Span;
use quote::quote_spanned;

let span = Span::call_site();
let tokens = quote_spanned! { span =>
    let _guard = ();
};

assert!(tokens.to_string().contains("_guard"));
```

**When to use it:** for generated code that can fail to compile because of
something in the user's input — trait bounds, method calls on their types. Point
it at the span of the thing responsible, usually via `syn`'s `Spanned::span`.
Use plain `quote!` for everything else; a wrong span is worse than a call-site
one, because it misdirects.

#### Interpolation with `#`

`#var` inserts a value; `#(...)` repeats over an iterable.

```
use quote::quote;

let name = quote::format_ident!("total");
let values = [1u8, 2, 3];

let tokens = quote! {
    let #name = [ #(#values),* ];
};

assert_eq!(tokens.to_string(), "let total = [1u8 , 2u8 , 3u8] ;");
```

**When to use it:** the core of the crate. Any `ToTokens` value can be
interpolated, and a repetition may iterate anything `IntoIterator`. Two
repetitions in one `#(...)` step in lockstep, which is how you pair field names
with field types.

#### `format_ident!`

Builds an `Ident` with `format!`-style syntax.

```
use quote::{format_ident, quote};

let base = format_ident!("Widget");
let builder = format_ident!("{}Builder", base);
let field = format_ident!("field_{}", 3usize);

assert_eq!(builder.to_string(), "WidgetBuilder");
assert_eq!(field.to_string(), "field_3");

let tokens = quote! { struct #builder; };
assert_eq!(tokens.to_string(), "struct WidgetBuilder ;");
```

**When to use it:** deriving a new name from an existing one — `FooBuilder`,
`__private_foo`, `field_0`. It produces an identifier, so it is checked for
validity and interpolates as a name rather than as text; building the string and
hoping is how you get an invalid identifier at the expansion site instead.
`Ident::new(&s, span)` is the lower-level form when the span must be specific.

### Deciding what can be interpolated

#### `ToTokens`

The trait making a value interpolatable. Implemented for `syn`'s types, for
primitives and `str`, and for anything you implement it on.

```
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

struct Version(u8, u8);

impl ToTokens for Version {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (major, minor) = (self.0, self.1);
        tokens.extend(quote! { (#major, #minor) });
    }
}

let version = Version(1, 4);
let tokens = quote! { const VERSION: (u8, u8) = #version; };

assert_eq!(tokens.to_string(), "const VERSION : (u8 , u8) = (1u8 , 4u8) ;");
```

**When to use it:** implement it for your own type when it appears repeatedly in
generated code — a parsed config that becomes a literal, a version that becomes
a tuple. It saves converting at each interpolation site and keeps `quote!` blocks
readable.

#### `ToTokens::to_token_stream`

Turns a single value into a `TokenStream` without wrapping it in `quote!`.

```
use quote::ToTokens;
use syn::{parse_str, Type};

let ty: Type = parse_str("Option<u32>").unwrap();
let stream = ty.to_token_stream();

assert_eq!(stream.to_string(), "Option < u32 >");
```

**When to use it:** when you already hold one `ToTokens` value and want its
tokens — comparing two types, hashing a signature, or feeding one node into
something expecting a stream. `into_token_stream` is the by-value form and
avoids a clone when you're finished with the value.

#### `IdentFragment`

The trait governing what may be substituted into `format_ident!`.

```
use quote::{format_ident, IdentFragment};

fn suffixed<T: IdentFragment>(base: &str, suffix: T) -> proc_macro2::Ident {
    format_ident!("{}_{}", base, suffix)
}

assert_eq!(suffixed("field", 2usize).to_string(), "field_2");
assert_eq!(suffixed("field", "name").to_string(), "field_name");
```

**When to use it:** writing a helper generic over what goes into a name. It
differs from `Display` in the detail that matters here — an `Ident` that is raw
(`r#type`) contributes `type`, so the result is a usable identifier rather than
one with `r#` embedded in the middle.

### Assembling streams

#### `TokenStreamExt::append_all`

Extends a stream with every item of an iterator, each converted through
`ToTokens`.

```
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

let fields = ["a", "b"].map(|n| quote::format_ident!("{n}"));

let mut tokens = TokenStream::new();
tokens.append_all(fields.iter().map(|f| quote! { let #f = 0; }));

assert_eq!(tokens.to_string(), "let a = 0 ; let b = 0 ;");
```

**When to use it:** building a stream up across a loop with logic in it, where a
single `#(...)` repetition can't express the condition. Inside a `quote!` block,
prefer the repetition — it is shorter and keeps the shape visible.

#### `TokenStreamExt::append_separated`

The same, with a separator token between items.

```
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

let values = [1u8, 2, 3];
let mut tokens = TokenStream::new();
tokens.append_separated(values.iter(), quote! { , });

assert_eq!(tokens.to_string(), "1u8 , 2u8 , 3u8");
```

**When to use it:** comma-separated lists assembled outside a `quote!` block.
`append_terminated` puts the separator after every item including the last,
which is what you want for statements and struct fields rather than arguments.

#### Composing streams

A `TokenStream` is itself `ToTokens`, so pieces built separately nest by
interpolation.

```
use quote::quote;

let body = quote! { println!("running"); };
let attrs = quote! { #[inline] };

let function = quote! {
    #attrs
    fn run() {
        #body
    }
};

let out = function.to_string();
assert!(out.contains("# [inline]"));
assert!(out.contains("println !"));
```

**When to use it:** whenever a macro grows past one block. Build the parts in
functions returning `TokenStream`, then interpolate them — it keeps each piece
testable on its own, which is the main reason macros built on `proc-macro2` are
easier to work on than ones written against the compiler's `TokenStream`.

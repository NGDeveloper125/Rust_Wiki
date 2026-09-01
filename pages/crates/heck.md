---
title: "heck"
version: "0.5.0"
publisher: "srrrse (withoutboats), Jonas Platte (jplatte)"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-09-01"
summary: "Case conversion between the shapes identifiers take: `snake_case`, `camelCase`, `UpperCamelCase`, `kebab-case` and the shouty variants — with one consistent rule for where a word begins."
categories: ["text-processing", "codegen", "no-std"]
repository: "https://github.com/withoutboats/heck"
---

## Overview

Turning `XMLHttpRequest` into `xml_http_request` looks like a two-line problem
until you write it. Where does a word start? What happens to a run of capitals,
to digits, to an underscore that is already there? Get any of those wrong and a
derive macro generates a field name nobody expects.

`heck` is that problem solved once. It defines a single rule for word
boundaries and applies it to every conversion:

```
use heck::{ToSnakeCase, ToUpperCamelCase, ToKebabCase};

// Runs of capitals are handled: XML|Http|Request, not X|M|L|Http|Request.
assert_eq!("XMLHttpRequest".to_snake_case(), "xml_http_request");

// The input's own separators don't matter — only the words it contains.
assert_eq!("hello__world".to_snake_case(), "hello_world");
assert_eq!("user name".to_upper_camel_case(), "UserName");
assert_eq!("HTTPResponse".to_kebab_case(), "http-response");
```

**The rule is worth reading once**, because everything else follows from it. A
word boundary sits at any non-alphanumeric character, and also inside a run of
letters: before an uppercase character followed by lowercase, and between the
last uppercase of a run and the lowercase that follows. So `HelloWorld` splits
`Hello|World` while `XMLHttpRequest` splits `XML|Http|Request`. Separators in
the input are discarded rather than preserved — adjacent ones fold into one, and
leading or trailing ones are dropped.

This is why it shows up as a build dependency almost everywhere: `serde`'s
`rename_all`, `clap`'s derive, and most code generators need exactly this, and
they need every crate to agree on the answer. Its download rank is that, not
direct use.

**It converts identifiers, not prose.** Unicode is handled, but the rule is
about programming casing rather than natural language — `to_title_case` will
capitalise every word rather than following an English style guide's rules about
short prepositions.

Each conversion comes in two forms: a `To*` trait method returning a `String`,
and an `As*` wrapper that implements `Display` and allocates nothing.

The crate is tiny — no dependencies, `#![no_std]` with `alloc`, MSRV 1.56 — and
**stable rather than actively developed**: the last release was 0.5.0 in March
2024, because a solved problem needs no releases. The repository is not archived
and still receives commits. `convert_case` is the alternative worth knowing,
offering configurable boundary rules and more cases at the cost of a larger API.

## When to use it

### Use case: Naming generated code

A derive macro reads a Rust identifier and has to emit a different one — a
builder's method, a getter, a companion type. The input casing is whatever the
user wrote.

```
use heck::{ToSnakeCase, ToUpperCamelCase};

/// Given a struct name, name its generated builder and the module holding it.
fn generated_names(type_name: &str) -> (String, String) {
    (
        format!("{}Builder", type_name.to_upper_camel_case()),
        type_name.to_snake_case(),
    )
}

assert_eq!(
    generated_names("HTTPClient"),
    ("HttpClientBuilder".to_string(), "http_client".to_string()),
);
assert_eq!(
    generated_names("user_account"),
    ("UserAccountBuilder".to_string(), "user_account".to_string()),
);
```

**Why it fits:** both inputs land on the same answer regardless of how they were
written, which is what stops a macro producing `HTTPClientBuilder` for one user
and `HttpClientBuilder` for another. Pair it with
[`quote`](quote.md)'s `format_ident!` to turn the result into a real identifier.

### Use case: Mapping Rust names onto an external convention

A JSON API wants `camelCase`, a CLI wants `kebab-case`, an environment variable
wants `SCREAMING_SNAKE_CASE`. One Rust name feeds all three.

```
use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase};

let field = "max_retry_count";

assert_eq!(field.to_lower_camel_case(), "maxRetryCount"); // JSON body
assert_eq!(field.to_kebab_case(), "max-retry-count");     // --max-retry-count
assert_eq!(field.to_shouty_snake_case(), "MAX_RETRY_COUNT"); // env var
```

**Why it fits:** the mapping is derived rather than written out, so adding a
field doesn't mean adding three strings that can disagree. This is exactly what
`serde`'s `rename_all` and `clap`'s derive do internally.

### Use case: Rendering a converted name without allocating

Inside a loop or a `Display` impl, the `As*` wrappers write straight into the
formatter.

```
use heck::AsSnakeCase;
use std::fmt::Write;

let fields = ["firstName", "lastName", "emailAddress"];

let mut sql = String::from("SELECT ");
for (i, field) in fields.iter().enumerate() {
    if i > 0 {
        sql.push_str(", ");
    }
    // No intermediate String per field.
    write!(sql, "{}", AsSnakeCase(field)).unwrap();
}

assert_eq!(sql, "SELECT first_name, last_name, email_address");
```

**Why it fits:** `to_snake_case()` would allocate a `String` per field only to
copy it into `sql` and drop it. The wrapper is the same conversion with the
allocation removed, which matters in a code generator running over thousands of
names.

## API map

Every case comes as a pair: a `To*` trait whose method returns a `String`, and
an `As*` tuple struct implementing `Display`. The traits are implemented for
`str`, so `use heck::ToSnakeCase;` puts `.to_snake_case()` on every string.

### Lowercase-separated

#### `ToSnakeCase`

`lower_case_with_underscores` — Rust's convention for functions, fields and
variables.

```
use heck::ToSnakeCase;

assert_eq!("HelloWorld".to_snake_case(), "hello_world");
assert_eq!("XMLHttpRequest".to_snake_case(), "xml_http_request");
assert_eq!("hello-world".to_snake_case(), "hello_world");
assert_eq!("HELLO".to_snake_case(), "hello");
```

**When to use it:** generating Rust identifiers, database column names, and JSON
keys for APIs that use snake case. `ToSnekCase` is an alias for the same thing,
kept as a joke rather than a distinction.

#### `ToKebabCase`

`lower-case-with-hyphens` — CLI flags, URL slugs, CSS.

```
use heck::ToKebabCase;

assert_eq!("MaxRetryCount".to_kebab_case(), "max-retry-count");
assert_eq!("max_retry_count".to_kebab_case(), "max-retry-count");
```

**When to use it:** long-form command-line options, file names and anything
appearing in a URL path. It is `snake_case` with a different separator, so the
two convert between each other losslessly.

### Uppercase-separated

#### `ToShoutySnakeCase`

`UPPER_CASE_WITH_UNDERSCORES` — constants and environment variables.

```
use heck::ToShoutySnakeCase;

assert_eq!("databaseUrl".to_shouty_snake_case(), "DATABASE_URL");
assert_eq!("HttpTimeout".to_shouty_snake_case(), "HTTP_TIMEOUT");
```

**When to use it:** naming a generated `const` or `static`, and deriving
environment variable names from config fields — the pairing that makes
`DATABASE_URL` map to a `database_url` field automatically.

#### `ToShoutyKebabCase`

`UPPER-CASE-WITH-HYPHENS`.

```
use heck::ToShoutyKebabCase;

assert_eq!("contentType".to_shouty_kebab_case(), "CONTENT-TYPE");
```

**When to use it:** rarely — some header conventions and legacy formats use it.
Note that HTTP header names are conventionally `Train-Case` rather than this, so
check before assuming.

### Camel and title

#### `ToUpperCamelCase`

`UpperCamelCase` — Rust types, traits and enum variants.

```
use heck::ToUpperCamelCase;

assert_eq!("user_account".to_upper_camel_case(), "UserAccount");
assert_eq!("http-client".to_upper_camel_case(), "HttpClient");
assert_eq!("XMLHttpRequest".to_upper_camel_case(), "XmlHttpRequest");
```

**When to use it:** generating type names from anything else. `ToPascalCase` is
an alias, for readers who know the convention by that name. Note the third
assertion — a run of capitals is normalised, so this is not an identity function
on names that already look like types.

#### `ToLowerCamelCase`

`lowerCamelCase` — JavaScript, and most JSON APIs.

```
use heck::ToLowerCamelCase;

assert_eq!("first_name".to_lower_camel_case(), "firstName");
assert_eq!("HTTP_TIMEOUT".to_lower_camel_case(), "httpTimeout");
```

**When to use it:** producing JSON keys or JavaScript identifiers from Rust
names. This is what `#[serde(rename_all = "camelCase")]` applies, so using it
directly gives the same answer serde would.

#### `ToTitleCase`

`Space Separated With Capitals` — for display rather than for code.

```
use heck::ToTitleCase;

assert_eq!("user_account".to_title_case(), "User Account");
assert_eq!("XMLHttpRequest".to_title_case(), "Xml Http Request");
```

**When to use it:** turning a field name into a label for a form or a table
header. It capitalises every word, so it is not English title case — "the" and
"of" get capitals too, and a human-written label beats it wherever one is
available.

#### `ToTrainCase`

`Capitalised-Words-With-Hyphens` — HTTP header names.

```
use heck::ToTrainCase;

assert_eq!("content_type".to_train_case(), "Content-Type");
assert_eq!("xForwardedFor".to_train_case(), "X-Forwarded-For");
```

**When to use it:** generating header names. HTTP treats them
case-insensitively, so this is a readability convention rather than a
requirement — but it is the one every tool prints.

### Formatting without allocating

#### The `As*` wrappers

Each case has a tuple struct implementing `Display`, converting as it writes.

```
use heck::{AsKebabCase, AsShoutySnakeCase, AsUpperCamelCase};

// Interpolated directly, with no String in between.
assert_eq!(format!("--{}", AsKebabCase("maxRetries")), "--max-retries");
assert_eq!(format!("{}", AsUpperCamelCase("user_id")), "UserId");
assert_eq!(format!("{}=1", AsShoutySnakeCase("logLevel")), "LOG_LEVEL=1");
```

**When to use it:** inside `format!`, `write!` and `Display` impls, and in loops
over many names. The `To*` method is clearer when you need an owned `String`
anyway; the wrapper is the one to reach for when the result goes straight into
another string.

---
title: "toml"
version: "1.1.8+spec-1.1.0"
publisher: "Ed Page (epage), Maintainers"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-18"
summary: "Reads and writes TOML through serde, so a configuration file deserialises straight into your own struct. Its parse errors point at the offending line with a caret, which is most of why you would pick it."
domain: "Serialization & data formats"
categories: ["parsing", "encoding", "config"]
repository: "https://github.com/toml-rs/toml"
docs: "https://docs.rs/toml/latest/toml/"
---

## Overview

TOML is the configuration format Cargo reads, and this is the crate that reads
it. It is a [`serde`](serde.md) data format, so the common case is one line:

```
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    name: String,
    port: u16,
}

let config: Config = toml::from_str(
    r#"
    name = "service"
    port = 8080
    "#,
)
.unwrap();

assert_eq!(config.name, "service");
assert_eq!(config.port, 8080);
```

Everything `serde` gives you applies — `#[serde(default)]`, `rename`, `flatten`,
nested structs, `Option` for optional keys — so there is little to learn beyond
the format itself.

**What sets it apart is the error reporting**, and it is worth seeing before
anything else:

```
use toml::Table;

let error = "name = \nport = 1".parse::<Table>().unwrap_err();

// Display is a rustc-style diagnostic, not a one-line message.
let shown = error.to_string();
assert!(shown.contains("TOML parse error at line 1, column 8"));
assert!(shown.contains('^'));

// And the parts are available separately.
assert!(error.message().contains("string values must be quoted"));
assert!(error.span().is_some());
```

For a configuration file that is close to the whole value proposition. A parse
failure that says *where* turns a support question into something the user fixes
themselves, and `message()` plus `span()` let you fold the position into your
own diagnostics.

**The boundary to understand: this crate does not preserve formatting.** Parsing
and re-serialising discards comments, blank lines and alignment, and reorders
keys:

```
use toml::Table;

let original = "# a comment\nname = \"x\"\n\nport   =   1\n";
let round_tripped = toml::to_string(&original.parse::<Table>().unwrap()).unwrap();

assert_eq!(round_tripped, "name = \"x\"\nport = 1\n");
assert!(!round_tripped.contains('#')); // the comment is gone
```

That is correct behaviour for a `serde` format — a `Value` has nowhere to keep a
comment — but it means you must not use this to *edit* a file a human
maintains. `toml_edit` is the crate for that: it keeps a document model with the
original formatting intact, which is how `cargo add` modifies your
`Cargo.toml` without reflowing it. Read and write whole files here; edit in
place there.

Two smaller behaviours worth knowing. **Keys come out sorted**, because `Table`
is a `BTreeMap` by default — enable the `preserve_order` feature to get document
order instead, backed by [`indexmap`](indexmap.md). And the version number
carries `+spec-1.1.0` as SemVer build metadata, naming the TOML specification
version it implements; it is informational, and as the [`semver`](semver.md)
page notes, build metadata is ignored in precedence comparisons.

It is maintained alongside Cargo's tooling by Ed Page, parses with
[`winnow`](winnow.md), and has optional `preserve_order` and `fast_hash`
features ([`foldhash`](foldhash.md) for the latter). `no_std` works by turning
off the default `std` feature. Note the **MSRV is 1.85 with edition 2024**,
which is high enough to matter on a pinned toolchain.

## When to use it

### Use case: Loading an application's configuration file

The reason most projects depend on it.

```
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Config {
    name: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default)]
    features: Vec<String>,
    database: Database,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Database {
    url: String,
}

fn default_port() -> u16 {
    8080
}

let text = r#"
name = "service"
features = ["metrics"]

[database]
url = "postgres://localhost"
"#;

let config: Config = toml::from_str(text).unwrap();

assert_eq!(config.port, 8080); // absent, so the default applied
assert_eq!(config.features, vec!["metrics".to_string()]);
assert_eq!(config.database.url, "postgres://localhost");
```

**Why it fits:** the struct *is* the schema. Every absent-but-required key, wrong
type and unparseable value is reported by the deserializer rather than by code
you wrote, and `#[serde(default)]` expresses optionality without `Option`
everywhere. Add `#[serde(deny_unknown_fields)]` to catch typos in key names,
which is usually what you want for a config file — silently ignoring
`prot = 8080` is a bad afternoon.

### Use case: Reporting exactly where a config file is wrong

Turning a parse or validation failure into something actionable.

```
use serde::Deserialize;
use toml::Spanned;

#[derive(Deserialize)]
struct Config {
    // Spanned records where in the source this value came from.
    name: Spanned<String>,
}

let text = "name = \"\"";
let config: Config = toml::from_str(text).unwrap();

// The value is empty, which parses fine but is not valid for us.
assert!(config.name.as_ref().is_empty());

// So point at it: the span is a byte range into the original text.
let span = config.name.span();
assert_eq!(&text[span.clone()], "\"\"");
assert_eq!(span, 7..9);
```

**Why it fits:** parse errors come with a position for free, but *semantic*
errors — an empty name, a port below 1024, a path that does not exist — happen
after deserialisation, when the source text is normally long forgotten.
`Spanned<T>` keeps the byte range, so your own validation can produce a
diagnostic as precise as the parser's. The cost is that the field type changes,
so reach for it on the fields you actually validate.

### Use case: Inspecting TOML whose shape you do not know

When there is no struct to deserialise into.

```
use toml::{Table, Value};

let text = r#"
title = "untyped"
count = 3

[section]
flag = true
"#;

// Table is a map; Value is any TOML value.
let table: Table = text.parse().unwrap();

assert_eq!(table["title"].as_str(), Some("untyped"));
assert_eq!(table["count"].as_integer(), Some(3));

// Navigation returns Option at every step.
let value: Value = Value::Table(table);
assert_eq!(
    value.get("section").and_then(|s| s.get("flag")).and_then(Value::as_bool),
    Some(true)
);
```

**Why it fits:** a linter, a formatter, a tool that merges two config files, or
a program accepting arbitrary user-defined keys has no fixed struct to target.
`Table` and `Value` are the untyped view — and they are also the escape hatch
when a file is *mostly* typed: deserialise the known part into a struct and keep
a `Table` for the rest with `#[serde(flatten)]`. The `as_*` methods all return
`Option`, which is the honest signature for a value whose type you have not
checked.

## API map

The crate is a `serde` format, so the surface is small: two functions each way,
two untyped types, and the error.

### Reading

#### `from_str`

Parses TOML into any `Deserialize` type.

```
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

let point: Point = toml::from_str("x = 1\ny = 2").unwrap();
assert_eq!(point, Point { x: 1, y: 2 });

// The target can equally be untyped.
let table: toml::Table = toml::from_str("x = 1").unwrap();
assert_eq!(table["x"].as_integer(), Some(1));
```

**When to use it:** for everything you read. `from_slice` takes `&[u8]` for a
file you have read as bytes, and `Table` additionally implements `FromStr`, so
`text.parse::<Table>()` works and reads better in a chain. There is no
`from_reader`: TOML needs the whole document to resolve tables, so read the file
to a `String` first.

#### `Spanned`

Keeps the source position of a deserialised value.

```
use serde::Deserialize;
use toml::Spanned;

#[derive(Deserialize)]
struct Entry {
    name: Spanned<String>,
    port: Spanned<u16>,
}

let text = "name = \"svc\"\nport = 80";
let entry: Entry = toml::from_str(text).unwrap();

// as_ref/into_inner reach the value; span() gives the byte range.
assert_eq!(entry.name.as_ref(), "svc");
assert_eq!(&text[entry.name.span()], "\"svc\"");
assert_eq!(entry.port.into_inner(), 80);
```

**When to use it:** on fields you validate yourself, so the error can point at
the right line. Note the span covers the raw source including quotes, which is
what a diagnostic renderer wants. It is a wrapper, so `as_ref`, `get_ref` or
`into_inner` is needed to reach the value — worth applying selectively rather
than to every field.

### Writing

#### `to_string` and `to_string_pretty`

Serialising a Rust value to TOML.

```
use serde::Serialize;

#[derive(Serialize)]
struct Config {
    name: String,
    ports: Vec<u16>,
}

let config = Config { name: "svc".into(), ports: vec![80, 443] };

let compact = toml::to_string(&config).unwrap();
assert!(compact.contains("name = \"svc\""));

// pretty puts arrays across multiple lines.
let pretty = toml::to_string_pretty(&config).unwrap();
assert!(pretty.len() >= compact.len());
```

**When to use it:** writing a generated config, a lockfile, a snapshot of
settings. `to_string_pretty` is the better default for anything a human will
read, since it expands arrays rather than keeping them on one line. One
structural point inherited from TOML: a table header captures every key after
it, so scalars have to precede nested tables. The serializer handles this for
you by reordering, which means **the output field order may not match your
struct's** — a struct declaring a nested table first still emits its scalar
fields above it. Older versions of this crate errored here instead.

### Untyped values

#### `Table` and `Value`

The dynamic representation.

```
use toml::{Table, Value};

let mut table = Table::new();
table.insert("name".to_string(), Value::from("svc"));
table.insert("port".to_string(), Value::from(8080));

// Keys are sorted, because Table is a BTreeMap by default.
assert_eq!(table.keys().collect::<Vec<_>>(), vec!["name", "port"]);

// as_* accessors return Option, one per TOML type.
assert_eq!(table["name"].as_str(), Some("svc"));
assert_eq!(table["port"].as_integer(), Some(8080));
assert_eq!(table["port"].as_str(), None); // wrong type, not a panic
```

**When to use it:** when the shape is unknown, or for building TOML
programmatically without a struct. `Value` covers the format's seven types
(string, integer, float, boolean, datetime, array, table) with an `as_*` and
`is_*` method each. The sorted key order is the default worth remembering —
enable `preserve_order` if a generated file should keep the order you inserted
in.

#### `Datetime`

TOML's native date and time type.

```
use toml::Table;

let table: Table = "stamp = 1979-05-27T07:32:00Z".parse().unwrap();

// TOML has first-class datetimes, unlike JSON.
let datetime = table["stamp"].as_datetime().unwrap();
assert_eq!(datetime.to_string(), "1979-05-27T07:32:00Z");
assert!(datetime.date.is_some());
assert!(datetime.time.is_some());
```

**When to use it:** when reading timestamps from config. This is a genuine
advantage of TOML over JSON, where a datetime is a string by convention and
every reader parses it differently. The type is deliberately minimal — it holds
the parsed components and does no arithmetic — so convert it to a `chrono` or
`jiff` type if you need to compute with it.

### Errors

#### `toml::de::Error`

The diagnostic.

```
use toml::Table;

let error = "port = ".parse::<Table>().unwrap_err();

// Display renders line, column, the source line and a caret.
let rendered = error.to_string();
assert!(rendered.starts_with("TOML parse error at line 1, column 8"));
assert!(rendered.contains('^'));

// Or take the parts and render them yourself.
assert!(!error.message().is_empty());
assert_eq!(error.span(), Some(7..7));
```

**When to use it:** print it with `{}` and stop there — the default rendering is
better than anything you would write, and users can act on it. Use `message()`
and `span()` when you are integrating into an existing diagnostic system that
owns the formatting, such as `annotate-snippets` or a language server. It
implements `std::error::Error`, so `?` into [`anyhow`](anyhow.md) or a
[`thiserror`](thiserror.md) enum works, though wrapping it in a terse message
often throws away the part that was useful.

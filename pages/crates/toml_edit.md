---
title: "toml_edit"
version: "0.25.17+spec-1.1.0"
publisher: "Ed Page (epage), ordian, Maintainers"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-29"
summary: "Parses TOML into a document model that keeps comments, spacing and key order, so you can change one value and write the file back otherwise untouched. It is how `cargo add` edits your Cargo.toml."
domain: "Serialization & data formats"
categories: ["parsing", "encoding", "config"]
repository: "https://github.com/toml-rs/toml"
docs: "https://docs.rs/toml_edit/latest/toml_edit/"
---

## Overview

Deserialising a TOML file into a struct and serialising it back is lossy by
nature: a struct field has nowhere to store the comment above it, so the comment
does not come back. That is fine for a file your program owns and fatal for a
file a person maintains. `toml_edit` solves the second case by parsing into a
document model that holds the formatting alongside the data.

```
use toml_edit::DocumentMut;

let original = r#"# what this package is
[package]
name    = "demo"      # aligned by hand
version = "0.1.0"

[dependencies]
serde = "1"
"#;

let doc = original.parse::<DocumentMut>().unwrap();

// Byte for byte: comments, alignment, blank lines, key order.
assert_eq!(doc.to_string(), original);
```

Round-tripping unchanged is the baseline. The point is to round-trip *changed*:

```
use toml_edit::{value, DocumentMut};

let original = r#"# what this package is
[package]
name    = "demo"      # aligned by hand
version = "0.1.0"

[dependencies]
serde = "1"
"#;

let mut doc = original.parse::<DocumentMut>().unwrap();
doc["package"]["version"] = value("0.2.0");

assert_eq!(
    doc.to_string(),
    r#"# what this package is
[package]
name    = "demo"      # aligned by hand
version = "0.2.0"

[dependencies]
serde = "1"
"#
);
```

One line changed and nothing else moved. This is what `cargo add` does to your
`Cargo.toml`, and it is not a coincidence — `cargo` depends on this crate.

**The unit of formatting is the key/value pair, and assigning replaces it.**
Worth understanding early, because it surprises people: the file at large is
preserved, but the line you assign to gets default formatting, so its own
alignment and trailing comment go with the old value.

```
use toml_edit::{value, DocumentMut, Formatted, Value};

let original = "version =   \"0.1.0\"   # pinned for now\n";

// Assigning a new item replaces the formatting of that pair.
let mut blunt = original.parse::<DocumentMut>().unwrap();
blunt["version"] = value("0.2.0");
assert_eq!(blunt.to_string(), "version = \"0.2.0\"\n");

// Carrying the decor across keeps the spacing and the comment.
let mut careful = original.parse::<DocumentMut>().unwrap();
let slot = careful["version"].as_value_mut().unwrap();
let decor = slot.decor().clone();
*slot = Value::String(Formatted::new("0.2.0".to_string()));
*slot.decor_mut() = decor;
assert_eq!(careful.to_string(), "version =   \"0.2.0\"   # pinned for now\n");
```

**Where it sits next to [`toml`](toml.md).** They are sibling crates in the same
repository, by the same maintainer, sharing the same parser — not one built on
the other. The choice is by task, not by preference: `toml` is a
[`serde`](serde.md) data format, so a configuration file becomes your own struct
in one call and formatting is discarded; `toml_edit` gives you a tree of
`Table`, `Value` and `Item` that you navigate and mutate, and keeps the file
intact. Read a file you own with `toml`. Edit a file someone else wrote with
`toml_edit`. If you need both — deserialise for validation, edit for output —
`toml_edit` has its own `serde` feature and `de`/`ser` modules, so you need not
depend on both crates.

The cost is the API. Everything is an `Item` that might be a value, a table, an
array of tables or nothing, so reading a nested string means a chain of
`as_table()` and `as_str()` calls returning
[`Option`](../concepts/error-handling/option.md), and the type system cannot
tell you that `[dependencies]` is a table. That is the honest trade: you are
working with a TOML document rather than with your own types.

**Two things it does not preserve**, both documented upstream: the order of
dotted keys, and the absence of a trailing newline — a file ending without one
gains one on output.

```
use toml_edit::DocumentMut;

let doc = "a = 1".parse::<DocumentMut>().unwrap();
assert_eq!(doc.to_string(), "a = 1\n"); // <- newline added
```

It is **pre-1.0 at 0.25.x**, which for this author means breaking changes arrive
in minor releases; pin the minor version and read the changelog on upgrade.
Document order is held by [`indexmap`](indexmap.md), which is a non-optional
dependency; parsing is optional (`parse`, pulling `toml_parser` and
[`winnow`](winnow.md)) as is writing (`display`), both on by default, and
`serde` is off. It is std-only, with **MSRV 1.85 and edition 2024**. As on the
[`toml`](toml.md) page, the `+spec-1.1.0` in the version is SemVer build
metadata naming the TOML specification implemented, and — as
[`semver`](semver.md) notes — is ignored in version comparisons.

## When to use it

The common thread is that a human will read the file afterwards.

### Use case: Changing one setting in a file someone else maintains

The `cargo add` shape, and the reason most projects reach for this crate.

```
use toml_edit::{value, DocumentMut};

/// Point a dependency at a new version, leaving the rest of the manifest alone.
fn bump(manifest: &str, name: &str, version: &str) -> Option<String> {
    let mut doc = manifest.parse::<DocumentMut>().ok()?;

    // Go through the table, not through the Item: Item::get_mut vivifies a
    // missing key, so `get_mut("dependencies")?.get_mut(name)?` would report
    // success for a dependency that is not there — and then add it.
    let deps = doc.get_mut("dependencies")?.as_table_like_mut()?;
    let dep = deps.get_mut(name)?;

    match dep.as_table_like_mut() {
        // serde = { version = "1", features = ["derive"] }
        Some(table) => {
            table.insert("version", value(version));
        }
        // serde = "1"
        None => *dep = value(version),
    }

    Some(doc.to_string())
}

let manifest = r#"[dependencies]
# needed for the config loader
serde = { version = "1", features = ["derive"] }
log   = "0.4"
"#;

assert_eq!(
    bump(manifest, "serde", "1.1").unwrap(),
    r#"[dependencies]
# needed for the config loader
serde = { version = "1.1", features = ["derive"] }
log   = "0.4"
"#
);

// The other spelling, handled by the same function.
assert!(bump(manifest, "log", "0.4.20")
    .unwrap()
    .contains("log   = \"0.4.20\""));
assert!(bump(manifest, "absent", "1").is_none());
```

**Why it fits:** the comment survives, `log`'s hand-alignment survives, and the
features array inside the inline table is untouched — none of which a `serde`
round-trip can promise. Note the two spellings a dependency can take:
`as_table_like_mut()` is what lets one branch cover both `[dependencies.serde]`
and the inline form, and forgetting that case is the commonest bug in code like
this.

### Use case: Adding a section, with a comment explaining it

Generating configuration into a file a person will read and edit later.

```
use toml_edit::{table, value, DocumentMut};

let existing = r#"[package]
name = "demo"
"#;

let mut doc = existing.parse::<DocumentMut>().unwrap();

doc["profile"] = table();
doc["profile"]["release"] = table();
doc["profile"]["release"]["lto"] = value(true);

// An explanation above the header, so the next reader knows why.
doc["profile"]["release"]
    .as_table_mut()
    .unwrap()
    .decor_mut()
    .set_prefix("\n# link-time optimisation, set by the release script\n");

// [profile] has no keys of its own, so suppress its header.
doc["profile"].as_table_mut().unwrap().set_implicit(true);

assert_eq!(
    doc.to_string(),
    r#"[package]
name = "demo"

# link-time optimisation, set by the release script
[profile.release]
lto = true
"#
);
```

**Why it fits:** a tool that writes configuration should leave behind something
a person can maintain, and that means the comment is part of the output rather
than an afterthought. `set_implicit(true)` is the detail that makes it look
hand-written: without it you get a bare `[profile]` header above
`[profile.release]`, which is valid TOML that no human would type.

### Use case: Reporting where a problem is in the source

A linter or validator that points at the offending line rather than naming a key.

```
use toml_edit::Document;

let source = r#"[server]
port = "8080"
"#;

// Document borrows the source and keeps spans; DocumentMut discards them.
let doc = Document::parse(source).unwrap();
let port = &doc.as_table()["server"]["port"];

if port.as_integer().is_none() {
    let span = port.span().unwrap();
    let line = source[..span.start].lines().count();

    assert_eq!(&source[span], "\"8080\"");
    assert_eq!(line, 2);
}

// A parse failure reports position the same way.
let error = "a = \nb = 1".parse::<toml_edit::DocumentMut>().unwrap_err();
assert!(error
    .to_string()
    .contains("TOML parse error at line 1, column 5"));
assert!(error.message().contains("string values must be quoted"));
assert_eq!(error.span(), Some(4..4));
```

**Why it fits:** "expected an integer for `server.port`" makes the user search
the file; a line number and the exact text does not. The catch is which document
type you parse into, and it is easy to get wrong: `DocumentMut` is detached from
the source so every `span()` on it returns `None`, while `Document<S>` holds the
input and resolves spans against it. Parse into `Document` to diagnose, then
call `into_mut()` if you also want to edit.

## API map

Everything hangs off a document that derefs to a `Table`, whose entries are
`Item`s. The groups below follow what you are trying to do to one.

### Parsing and writing

#### `DocumentMut`

The editable document, and what `FromStr` gives you.

```
use toml_edit::DocumentMut;

let mut doc = "a = 1\n".parse::<DocumentMut>().unwrap();

// It derefs to Table, so the Table API is available directly.
assert!(doc.contains_key("a"));
assert_eq!(doc.len(), 1);

// And an empty one, for generating a file from nothing.
let fresh = DocumentMut::new();
assert_eq!(fresh.to_string(), "");

doc.set_trailing("# end of generated section\n");
assert_eq!(doc.to_string(), "a = 1\n# end of generated section\n");
```

**When to use it:** for anything that edits. `to_string()` is the whole output
step — there is no separate serialiser to call. `set_trailing` controls what
follows the last item, which is where a trailing comment at the bottom of a file
lives; it is the one piece of decor that belongs to the document rather than to a
key.

#### `Document`

The immutable document, which keeps the source and therefore the spans.

```
use toml_edit::Document;

let source = "[a]\nkey = 1\n";
let doc = Document::parse(source).unwrap();

assert_eq!(doc.raw(), source);
assert_eq!(doc.as_table()["a"].span(), Some(0..3));
assert_eq!(doc.as_table()["a"]["key"].span(), Some(10..11));

// Convert when you want to start editing; spans are dropped.
let editable = doc.into_mut();
assert_eq!(editable["a"]["key"].span(), None);
```

**When to use it:** whenever you need positions — diagnostics, a formatter, a
language server. `Document::parse` borrows (`Document<&str>`) or owns
(`Document<String>` via `parse::<Document<String>>()`), and `into_mut` gives up
the source for mutability. If you are only going to edit, skip it and parse
straight into `DocumentMut`. `ImDocument` is a deprecated alias for this type
from before it was renamed.

#### `TomlError`

What a failed parse gives you.

```
use toml_edit::DocumentMut;

let error = "[unclosed\n".parse::<DocumentMut>().unwrap_err();

// Display is a rustc-style diagnostic with a caret.
let shown = error.to_string();
assert!(shown.contains("TOML parse error at line 1"));
assert!(shown.contains('^'));

// The parts, for folding into your own reporting.
assert!(!error.message().is_empty());
assert!(error.span().is_some());
```

**When to use it:** print it with `Display` and you are done — the default
rendering already names the line, column and problem. Reach for `message()` and
`span()` only when you are building the diagnostic yourself, for instance to
attach a file name and your own labels.

### Navigating the tree

#### Indexing with `[]`

The short path to a nested item, and the gotcha that comes with it.

```
use toml_edit::{value, DocumentMut};

let doc = "[a]\nb = \"x\"\n".parse::<DocumentMut>().unwrap();
assert_eq!(doc["a"]["b"].as_str(), Some("x"));

// Immutable indexing of a missing key panics, so prefer get for lookups.
assert!(doc.get("nope").is_none());

// Mutable indexing creates the path as INLINE tables, not headers.
let mut fresh = DocumentMut::new();
fresh["a"]["b"]["c"] = value(1);
assert_eq!(fresh.to_string(), "a = { b = { c = 1 } }\n");

// Item::get_mut vivifies too, so it never reports a key as absent.
let mut probe = "[a]\nb = \"x\"\n".parse::<DocumentMut>().unwrap();
assert!(probe.get_mut("a").unwrap().get_mut("missing").is_some()); // <- not None
// The table's own get_mut answers the question honestly.
assert!(probe["a"].as_table_mut().unwrap().get_mut("missing").is_none());
```

**When to use it:** for a path you know exists, in a test or after checking.
Three traps, and they are the main reason to read this entry. Indexing a missing
key **panics** on the immutable side, so use `get` and `get_mut` in anything that
handles untrusted input. Mutable indexing vivifies the missing levels as *inline*
tables — if you wanted `[a.b]` headers, build them with `table()` instead, as in
the second use case above. And `Item::get_mut` vivifies as well, which makes it
unsafe for existence checks: a `?` chain through it succeeds on a key that is not
in the file. Go through `as_table_like_mut()` and use the table's `get_mut`, as
the first use case does, whenever the answer "not present" matters.

#### `Item`

The enum every entry in a table is.

```
use toml_edit::DocumentMut;

let doc = "n = 1\ns = 'x'\n[t]\nk = 2\n[[aot]]\n"
    .parse::<DocumentMut>()
    .unwrap();

// type_name is the quickest way to see what you have.
assert_eq!(doc["n"].type_name(), "integer");
assert_eq!(doc["t"].type_name(), "table");
assert_eq!(doc["aot"].type_name(), "array of tables");

// The as_* family returns Option, and shortcuts through Value.
assert_eq!(doc["n"].as_integer(), Some(1));
assert_eq!(doc["s"].as_str(), Some("x"));
assert!(doc["t"].as_table().is_some());
assert!(doc["n"].as_table().is_none());

// get_values walks the values in a table, dotted keys included.
let dotted = "[t]\na = 1\nx.y = 2\n".parse::<DocumentMut>().unwrap();
let paths: Vec<Vec<&str>> = dotted["t"]
    .as_table()
    .unwrap()
    .get_values()
    .iter()
    .map(|(keys, _)| keys.iter().map(|k| k.get()).collect())
    .collect();
assert_eq!(paths, vec![vec!["a"], vec!["x", "y"]]);
```

**When to use it:** constantly — it is the type you are always holding.
`type_name()` is underrated for error messages, since "expected a table, found
integer" writes itself. Note what `get_values` does *not* include: sub-tables
with their own headers. It returns the table's own values and its dotted keys,
so walking a whole document means recursing or using `visit_mut`.

#### `TableLike`

The trait over `Table` and `InlineTable`.

```
use toml_edit::{DocumentMut, TableLike};

fn names(table: &dyn TableLike) -> Vec<&str> {
    table.iter().map(|(key, _)| key).collect()
}

let doc = "[a]\nx = 1\n[b]\ninline = { x = 1 }\n"
    .parse::<DocumentMut>()
    .unwrap();

assert_eq!(names(doc["a"].as_table().unwrap()), vec!["x"]);
assert_eq!(names(doc["b"]["inline"].as_inline_table().unwrap()), vec!["x"]);

// as_table_like covers both without caring which it is.
assert_eq!(doc["b"]["inline"].as_table_like().unwrap().len(), 1);
```

**When to use it:** whenever the same key could be written either way, which in
a human-maintained file is most of the time — `[dependencies.serde]` and
`serde = { ... }` mean the same thing and are different types. Writing against
`as_table_like_mut()` handles both; checking only `as_table_mut()` is a bug that
shows up the first time a user writes the inline form.

### Changing values

#### `value`

Wraps a Rust value as an `Item` with default formatting.

```
use toml_edit::{value, Array, DocumentMut, Value};

let mut doc = DocumentMut::new();

doc["s"] = value("text");
doc["n"] = value(42);
doc["f"] = value(1.5);
doc["b"] = value(true);

let mut arr = Array::new();
arr.push("a");
arr.push("b");
doc["list"] = value(Value::Array(arr));

assert_eq!(
    doc.to_string(),
    "s = \"text\"\nn = 42\nf = 1.5\nb = true\nlist = [\"a\", \"b\"]\n"
);
```

**When to use it:** for every assignment of a scalar or array, which is the bulk
of what editing a config file involves. It takes anything `Into<Value>`, so the
primitives, `String`, `&str`, `Array` and `InlineTable` all work. Remember that
it brings default formatting with it — see `Decor` for keeping the old
formatting, and the next entry for choosing a different one.

#### Choosing how a value is written

`Formatted` holds a value's logical content and its source text separately.

```
use toml_edit::{DocumentMut, Item, Value};

// The original spelling is preserved and readable.
let doc = "hex = 0xFF\nbig = 1_000\nlit = 'raw'\n"
    .parse::<DocumentMut>()
    .unwrap();

assert_eq!(doc["hex"].as_integer(), Some(255));
match doc["hex"].as_value().unwrap() {
    Value::Integer(f) => assert_eq!(f.display_repr(), "0xFF"),
    _ => unreachable!(),
}

// Parsing an Item picks the representation for a new value.
let mut out = DocumentMut::new();
out["lit"] = "'single quoted'".parse::<Item>().unwrap();
out["hex"] = "0xFF".parse::<Item>().unwrap();
assert_eq!(out.to_string(), "lit = 'single quoted'\nhex = 0xFF\n");
```

**When to use it:** `display_repr()` when you need the text as written — showing
a user their own value back, or deciding whether a rewrite is actually a change.
Parsing an `Item` or a `Value` from a string is the way to produce a
representation `value()` would not: a literal string, a hex or underscored
integer, a multi-line string. Setting `0xFF` through `value(255)` gives you
`255`, which is correct and not what the file said.

### Changing structure

#### `table` and `set_implicit`

A new table with its own header.

```
use toml_edit::{table, value, DocumentMut};

let mut doc = DocumentMut::new();

doc["a"] = table();
doc["a"]["b"] = table();
doc["a"]["b"]["key"] = value(1);

// [a] holds nothing itself, so its header is noise.
assert_eq!(doc.to_string(), "[a]\n\n[a.b]\nkey = 1\n");

doc["a"].as_table_mut().unwrap().set_implicit(true);
assert_eq!(doc.to_string(), "[a.b]\nkey = 1\n");
```

**When to use it:** `table()` whenever you want a `[header]` rather than an
inline table — which, as the indexing entry notes, is not what mutable indexing
gives you by default. Call `set_implicit(true)` on every intermediate table you
create, because a parent that only exists to hold a child should not print a
header. A parsed document already has this set correctly on its own implicit
tables.

#### `array` and `ArrayOfTables`

The `[[section]]` form.

```
use toml_edit::{array, value, DocumentMut, Table};

let mut doc = DocumentMut::new();
doc["bin"] = array();

let tables = doc["bin"].as_array_of_tables_mut().unwrap();
for name in ["server", "client"] {
    let mut entry = Table::new();
    entry["name"] = value(name);
    tables.push(entry);
}

assert_eq!(tables.len(), 2);
assert_eq!(
    doc.to_string(),
    "[[bin]]\nname = \"server\"\n\n[[bin]]\nname = \"client\"\n"
);

// Indexing is positional, and removal shifts the rest.
let removed = doc["bin"].as_array_of_tables_mut().unwrap().remove(0);
assert_eq!(removed["name"].as_str(), Some("server"));
```

**When to use it:** for repeated sections — `[[bin]]`, `[[test]]`, a list of
servers. Note that `array()` returns an *array of tables*, not an array of
values; the confusingly-similar `Array` below is the `x = [1, 2]` kind. Mixing
them up produces a type error rather than wrong output, which is the one mercy
here.

#### `Array`

The `x = [1, 2]` kind.

```
use toml_edit::{Array, DocumentMut};

let mut doc = "features = [\"a\"]\n".parse::<DocumentMut>().unwrap();
doc["features"].as_array_mut().unwrap().push("b");
assert_eq!(doc.to_string(), "features = [\"a\", \"b\"]\n");

let features = doc["features"].as_array_mut().unwrap();
features.sort_by(|a, b| b.as_str().cmp(&a.as_str()));
features.retain(|v| v.as_str() != Some("a"));
assert_eq!(doc.to_string(), "features = [\"b\"]\n");

// Built from scratch, it formats on one line.
let mut fresh = Array::new();
fresh.extend(["x", "y"]);
assert_eq!(fresh.to_string(), "[\"x\", \"y\"]");
```

**When to use it:** for editing a feature list, a path list, an exclude list.
`push` picks up default formatting and inserts it after the existing items, which
is usually right; `push_formatted` takes a `Value` whose decor you have already
set, for matching an array that is laid out one item per line. `sort_by`,
`retain` and `replace` behave as the `Vec` methods of the same names.

#### `InlineTable`

The `x = { a = 1 }` kind.

```
use toml_edit::{table, Array, DocumentMut, InlineTable, Item, Value};

let mut inline = InlineTable::new();
inline.insert("version", "1.0".into());
inline.insert("features", {
    let mut features = Array::new();
    features.push("derive");
    Value::Array(features)
});

let mut doc = DocumentMut::new();
doc["dependencies"] = table();
doc["dependencies"]["serde"] = Item::Value(Value::InlineTable(inline));

assert_eq!(
    doc.to_string(),
    "[dependencies]\nserde = { version = \"1.0\", features = [\"derive\"] }\n"
);

// A header table can be converted into one, and back.
let parsed = "[a]\nx = 1\n".parse::<DocumentMut>().unwrap();
let flattened = parsed["a"].as_table().unwrap().clone().into_inline_table();
assert_eq!(flattened.to_string(), "{ x = 1 }");
```

**When to use it:** for the compact form a dependency specification usually
takes, and for any value that belongs on one line. `into_table()` and
`into_inline_table()` convert between the two shapes, which is how you turn
`serde = "1"` into a `[dependencies.serde]` block when it grows too long.
`set_trailing_comma` and `fmt` control the punctuation.

#### `Entry`

Insert-if-absent, without overwriting what is there.

```
use toml_edit::{value, DocumentMut};

let mut doc = "[t]\nkept = 1\n".parse::<DocumentMut>().unwrap();
let table = doc["t"].as_table_mut().unwrap();

// Leaves an existing value alone.
table.entry("kept").or_insert(value(99));
// Adds a missing one.
table.entry("added").or_insert(value(2));

assert_eq!(doc.to_string(), "[t]\nkept = 1\nadded = 2\n");
```

**When to use it:** for defaults — filling in a key a user may already have set.
It is the distinction between `insert`, which overwrites and returns the old
item, and `entry().or_insert()`, which does not. For a tool that edits someone
else's file this is almost always the one you want, since silently replacing a
value the user chose is the behaviour people file bugs about.

### Controlling layout

#### `Decor`

The whitespace and comments around an item.

```
use toml_edit::{value, DocumentMut};

let mut doc = DocumentMut::new();
doc["port"] = value(8080);

// A prefix on the key is the comment above the line.
doc.key_mut("port")
    .unwrap()
    .leaf_decor_mut()
    .set_prefix("# which port to bind\n");

// A suffix on the value is the comment after it.
doc["port"]
    .as_value_mut()
    .unwrap()
    .decor_mut()
    .set_suffix("  # changed in 2.0");

assert_eq!(
    doc.to_string(),
    "# which port to bind\nport = 8080  # changed in 2.0\n"
);
```

**When to use it:** to write a comment, to keep one across an assignment (as in
the overview), or to control blank lines between sections by putting `\n` in a
prefix. The two sides are worth remembering: a comment *above* a key is that
key's prefix, a comment *after* a value is that value's suffix. One consequence
follows directly — removing a key removes its prefix, so the comment explaining
it disappears with it, which is usually what you want and occasionally not.

#### `Key` and `insert_formatted`

Inserting with a chosen key representation.

```
use toml_edit::{value, DocumentMut, Key};

let mut doc = DocumentMut::new();

// A key needing quotes gets them.
doc.insert_formatted(&Key::new("needs quotes"), value(1));
assert_eq!(doc.to_string(), "\"needs quotes\" = 1\n");

// A dotted path parses into its parts.
let parts = Key::parse("a.b").unwrap();
assert_eq!(
    parts.iter().map(|k| k.get()).collect::<Vec<_>>(),
    vec!["a", "b"]
);

// And the key as written is readable on a parsed document.
let parsed = "'quoted' = 1\n".parse::<DocumentMut>().unwrap();
assert_eq!(parsed.key("quoted").unwrap().display_repr(), "'quoted'");
```

**When to use it:** `insert_formatted` when the key needs its own formatting —
quoting, or a comment carried on a `Key` built with `with_leaf_decor`. Plain
`insert` takes a `&str` and quotes it only when TOML requires it, which is the
right default. `Key::parse` is how you split a user-supplied `a.b.c` into the
path to walk, and it validates the syntax on the way.

#### `sort_values` and `sort_values_by`

Reordering a table's keys.

```
use toml_edit::DocumentMut;

let mut doc = "[deps]\nserde = \"1\"\nanyhow = \"1\"\nlog = \"0.4\"\n"
    .parse::<DocumentMut>()
    .unwrap();

doc["deps"].as_table_mut().unwrap().sort_values();
assert_eq!(
    doc.to_string(),
    "[deps]\nanyhow = \"1\"\nlog = \"0.4\"\nserde = \"1\"\n"
);
```

**When to use it:** for the "keep the dependency list alphabetical" job, which is
most of why it exists. It sorts the key/value pairs and leaves sub-tables where
they are, and the comments travel with their keys because they are part of the
key's decor. `sort_values_by` takes a comparator over keys and items for any
other order — by value, or case-insensitively.

#### `fmt`

Normalise formatting — deliberately destructive.

```
use toml_edit::DocumentMut;

// On an array: collapses it onto one line.
let mut arr = "a = [\n  1,\n  2,\n]\n".parse::<DocumentMut>().unwrap();
arr["a"].as_array_mut().unwrap().fmt();
assert_eq!(arr.to_string(), "a = [1, 2]\n");

// On a table: resets spacing AND drops the comments.
let mut tbl = "[t]\nx   =   1   # why\n".parse::<DocumentMut>().unwrap();
tbl["t"].as_table_mut().unwrap().fmt();
assert_eq!(tbl.to_string(), "[t]\nx = 1\n");
```

**When to use it:** rarely, and never on a whole document you were asked to
preserve — it throws away exactly what you chose this crate for. The legitimate
uses are narrow: tidying a subtree you just generated, or collapsing an inline
table after building it up, which is what the crate's own example does. If what
you want is "normalise this file", round-tripping through [`toml`](toml.md) is
simpler and more honest about what it does.

#### `set_position` and `set_dotted`

Where a table goes, and which syntax it uses.

```
use toml_edit::{table, value, DocumentMut};

// Position is the table's index in document order.
let doc = "[b]\nx = 1\n\n[a]\ny = 2\n".parse::<DocumentMut>().unwrap();
assert_eq!(doc["b"].as_table().unwrap().position(), Some(1));
assert_eq!(doc["a"].as_table().unwrap().position(), Some(2));

// Dotted renders a table as `a.b = 1` instead of a header.
let mut dotted = DocumentMut::new();
dotted["a"] = table();
dotted["a"]["b"] = value(1);
dotted["a"].as_table_mut().unwrap().set_dotted(true);
assert_eq!(dotted.to_string(), "a.b = 1\n");
```

**When to use it:** `set_position` to place a table you are appending — a new
section at the end rather than wherever the tree happens to put it. Be aware the
table's decor moves with it, so a blank line that used to separate sections can
end up in the wrong place; check the output. `set_dotted` is for configuration
that reads better as a dotted key than as a header, and is the form `toml_edit`
cannot preserve the ordering of.

### Traversal and serde

#### `visit_mut::VisitMut`

A sweep over every node in a document.

```
use toml_edit::visit_mut::VisitMut;
use toml_edit::{DocumentMut, Formatted};

/// Rewrite every string in the document.
struct Rewrite;

impl VisitMut for Rewrite {
    fn visit_string_mut(&mut self, node: &mut Formatted<String>) {
        if node.value() == "OLD" {
            *node = Formatted::new("NEW".to_string());
        }
    }
}

let mut doc = "a = \"OLD\"\n[t]\nb = \"OLD\"\nc = [\"OLD\", \"keep\"]\n"
    .parse::<DocumentMut>()
    .unwrap();

Rewrite.visit_document_mut(&mut doc);

assert_eq!(
    doc.to_string(),
    "a = \"NEW\"\n[t]\nb = \"NEW\"\nc = [\"NEW\", \"keep\"]\n"
);
```

**When to use it:** for a change that applies everywhere rather than at a known
path — rewriting every `path` entry in a workspace, redacting every secret,
upgrading a renamed key throughout. The default method bodies recurse, so you
override only the node type you care about and the walk handles tables, inline
tables, arrays and arrays of tables for you. A `visit` module does the same
read-only, for collecting rather than changing.

#### `de` and `ser`

Serde support, so one dependency covers both jobs.

```
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Config {
    name: String,
    port: u16,
}

let config: Config = toml_edit::de::from_str("name = \"demo\"\nport = 8080\n").unwrap();
assert_eq!(
    config,
    Config {
        name: "demo".to_string(),
        port: 8080
    }
);

assert_eq!(
    toml_edit::ser::to_string(&config).unwrap(),
    "name = \"demo\"\nport = 8080\n"
);

// Or straight into a document, to keep editing.
let mut doc = toml_edit::ser::to_document(&config).unwrap();
doc["port"] = toml_edit::value(9090);
assert_eq!(doc.to_string(), "name = \"demo\"\nport = 9090\n");

// Deserialisation errors carry a position, as parse errors do.
let error =
    toml_edit::de::from_str::<Config>("name = \"x\"\nport = \"oops\"\n").unwrap_err();
assert!(error.to_string().contains("invalid type: string"));
assert_eq!(error.span(), Some(18..24));
```

**When to use it:** when a tool both validates and edits — deserialise into your
own struct to check the file makes sense, then edit the document to change it.
Requires the `serde` feature, which is off by default. If you are *only*
deserialising, the [`toml`](toml.md) crate is the more direct answer; this exists
so that needing both does not mean two TOML implementations in your dependency
tree. `ser::to_document` is the bridge between the two halves.

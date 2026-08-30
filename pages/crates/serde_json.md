---
title: "serde_json"
version: "1.0.151"
publisher: "David Tolnay (dtolnay), publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-30"
summary: "JSON for Rust: parse into your own types with `from_str`, write them back with `to_string`, or work with untyped `Value` when the shape isn't known ahead of time."
categories: ["serialization", "json", "no-std"]
repository: "https://github.com/serde-rs/json"
---

## Overview

`serde_json` is the JSON half of Serde. Serde defines how a Rust type is taken
apart and put back together; `serde_json` is the format that reads and writes
that as JSON. In practice you use them together — `serde` for the derive,
`serde_json` for the functions:

```
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    host: String,
    port: u16,
}

let text = r#"{"host":"example.com","port":8080}"#;

let config: Config = serde_json::from_str(text).unwrap();
assert_eq!(config.port, 8080);

let round_tripped = serde_json::to_string(&config).unwrap();
assert_eq!(round_tripped, text);
```

**The first decision is typed or untyped**, and it is the one that shapes
everything else:

- **Into your own struct.** The type states the schema, missing fields are
  errors, and the rest of your program works with real Rust types. This is the
  default and the right answer whenever you know what the JSON should look like.
- **Into `Value`.** An enum covering the six JSON kinds, for when the shape is
  genuinely unknown — a passthrough proxy, a config with user-defined sections,
  a debugging tool. Every access returns `Option`, because nothing is
  guaranteed.

Mixing them is normal: a struct with a `Value` field parses the known parts
strictly and keeps the unknown ones intact.

**What's worth knowing before you hit it:**

- **Numbers.** JSON has one number type; Rust has many. Integers beyond `i64`/`u64`
  and floats that don't round-trip need the `arbitrary_precision` and
  `float_roundtrip` features, and JSON has no way to express `NaN` or infinity —
  serialising one is an error, not a silent `null`.
- **Key order is lost by default.** `Value`'s object is a `BTreeMap`, so
  round-tripping a document reorders its keys alphabetically. The
  `preserve_order` feature swaps in [`indexmap`](indexmap.md) to keep file
  order, which matters for anything a human reads or a diff touches.
- **Depth is unbounded by default** in the sense that deeply nested input can
  overflow the stack while parsing. `unbounded_depth` exists for the opposite
  need; for untrusted input, cap the size before parsing.

It is the de facto standard, requires Rust 1.71, and works `no_std` with
`alloc`. Reach for `simd-json` only if profiling says parsing is your
bottleneck, and for `sonic-rs` under the same condition — both trade portability
and API familiarity for throughput.

## When to use it

### Use case: Reading a config or an API response into a struct

The type is the schema. Fields that don't parse are errors, and the code
downstream never handles JSON at all.

```
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    id: u64,
    name: String,
    #[serde(default)]
    tags: Vec<String>, // <- absent in the JSON becomes an empty Vec
}

let body = r#"{"id":7,"name":"widget"}"#;
let parsed: Response = serde_json::from_str(body).unwrap();

assert_eq!(parsed.id, 7);
assert!(parsed.tags.is_empty());

// A wrong type is an error, not a silent zero.
let bad = serde_json::from_str::<Response>(r#"{"id":"seven","name":"x"}"#);
assert!(bad.is_err());
```

**Why it fits:** validation happens once, at the boundary, and everything after
it is a `Response`. The `#[serde(default)]` attribute is the tool for optional
fields — without it a missing `tags` is an error, which is often what you want
too.

### Use case: Handling JSON whose shape you don't control

A proxy, a webhook receiver, or a tool that inspects arbitrary documents can't
name the type in advance. `Value` and JSON Pointer cover it.

```
let body = r#"{
    "event": "push",
    "repository": { "name": "wiki", "owner": { "login": "octocat" } }
}"#;

let event: serde_json::Value = serde_json::from_str(body).unwrap();

// Indexing is infallible but returns Null for anything missing.
assert_eq!(event["event"], "push");
assert_eq!(event["nope"], serde_json::Value::Null);

// Pointer walks a path and tells you when it isn't there.
let owner = event.pointer("/repository/owner/login").and_then(|v| v.as_str());
assert_eq!(owner, Some("octocat"));
assert_eq!(event.pointer("/repository/stars"), None);
```

**Why it fits:** the document survives untouched, including fields you know
nothing about, and the two access styles differ where it matters — `[]` is terse
and forgiving, `pointer` distinguishes "absent" from "present and null".

### Use case: Keeping unknown fields through a round trip

A service that reads a document, changes one field and writes it back must not
discard the parts it doesn't understand. `#[serde(flatten)]` into a map does
that.

```
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
struct Document {
    version: u32,
    #[serde(flatten)]
    rest: Map<String, Value>, // <- everything not named above
}

let input = r#"{"version":1,"title":"notes","custom":{"a":1}}"#;
let mut doc: Document = serde_json::from_str(input).unwrap();

doc.version = 2;

let output = serde_json::to_string(&doc).unwrap();
assert!(output.contains(r#""version":2"#));
assert!(output.contains(r#""title":"notes""#)); // <- preserved
assert!(output.contains(r#""custom":{"a":1}"#)); // <- preserved, nested and all
```

**Why it fits:** the fields you care about are typed and checked; the rest is
carried through verbatim. Parsing the whole thing as `Value` would work too, but
then `version` is a `Value` you have to unwrap at every use.

## API map

The crate is small at the top level: four ways in, four ways out, and the
`Value` type with its accessors. Anything about *how a type maps to JSON* —
renaming fields, defaults, flattening, enum representations — is `serde`'s
attributes rather than this crate's API.

### Parsing

#### `from_str`

Parses a `&str` into any `Deserialize` type.

```
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
struct Point { x: i32, y: i32 }

let p: Point = serde_json::from_str(r#"{"x":1,"y":2}"#).unwrap();
assert_eq!(p, Point { x: 1, y: 2 });

// Also into Value, when there is no type to name.
let v: serde_json::Value = serde_json::from_str(r#"[1,2,3]"#).unwrap();
assert_eq!(v[0], 1);
```

**When to use it:** the everyday entry point when the JSON is already in memory.
Borrowed types like `&str` fields can borrow from the input, which avoids
copying — that only works with `from_str` and `from_slice`, not `from_reader`.

#### `from_slice`

The same, from bytes.

```
use serde::Deserialize;

#[derive(Deserialize)]
struct Msg { ok: bool }

let bytes = br#"{"ok":true}"#;
let msg: Msg = serde_json::from_slice(bytes).unwrap();
assert!(msg.ok);
```

**When to use it:** HTTP bodies, files read as `Vec<u8>`, anything already
bytes. It skips the UTF-8 validation a `String` conversion would do first, and
reports invalid UTF-8 as a parse error like any other.

#### `from_reader`

Parses from anything implementing `Read`.

```
use serde::Deserialize;

#[derive(Deserialize)]
struct Item { n: u8 }

let source = std::io::Cursor::new(r#"{"n":5}"#);
let item: Item = serde_json::from_reader(source).unwrap();
assert_eq!(item.n, 5);
```

**When to use it:** a file or socket you'd rather not buffer entirely. Note the
trade — it cannot borrow from the input, so `&str` fields must become `String`,
and for data already in memory `from_slice` is faster because it avoids the
intermediate copies.

#### `Error` and its position

Parse failures carry a line and column.

```
let err = serde_json::from_str::<serde_json::Value>("{\"a\": 1,\n \"b\": }").unwrap_err();

assert_eq!(err.line(), 2);
assert!(err.is_syntax());
assert!(err.to_string().contains("line 2"));
```

**When to use it:** reporting a bad config back to whoever wrote it — the line
number is the difference between a usable message and "invalid JSON". The
`is_syntax`, `is_data` and `is_io` predicates separate malformed JSON from JSON
that is well-formed but doesn't match your type.

### Serialising

#### `to_string`

Any `Serialize` type to a compact JSON string.

```
use serde::Serialize;

#[derive(Serialize)]
struct Point { x: i32, y: i32 }

let json = serde_json::to_string(&Point { x: 1, y: 2 }).unwrap();
assert_eq!(json, r#"{"x":1,"y":2}"#);
```

**When to use it:** wire formats and storage, where nobody reads the bytes.
It returns `Result` because serialisation can fail — a map with non-string keys,
or a `f64` that is `NaN`.

#### `to_string_pretty`

The same, indented.

```
use serde::Serialize;

#[derive(Serialize)]
struct Point { x: i32 }

let json = serde_json::to_string_pretty(&Point { x: 1 }).unwrap();
assert_eq!(json, "{\n  \"x\": 1\n}");
```

**When to use it:** files people edit, CLI output, fixtures committed to git —
anywhere a diff should be readable. Two-space indent is fixed; `Serializer::with_formatter`
is the way to change it.

#### `to_writer`

Serialises straight into a `Write`, with no intermediate `String`.

```
use serde::Serialize;

#[derive(Serialize)]
struct Point { x: i32 }

let mut out = Vec::new();
serde_json::to_writer(&mut out, &Point { x: 1 }).unwrap();

assert_eq!(String::from_utf8(out).unwrap(), r#"{"x":1}"#);
```

**When to use it:** writing to a file or socket, especially for large documents
where holding the whole encoding in memory is wasteful. `to_vec` is the same
idea when you want the bytes rather than a `String`.

#### `json!`

Builds a `Value` from JSON-shaped literal syntax, interpolating Rust values.

```
use serde_json::json;

let name = "widget";
let count = 3;

let value = json!({
    "name": name,
    "count": count,
    "tags": ["a", "b"],
    "nested": { "ok": true }
});

assert_eq!(value["name"], "widget");
assert_eq!(value["tags"][1], "b");
assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"count":3,"name":"widget","nested":{"ok":true},"tags":["a","b"]}"#);
```

**When to use it:** tests, small request bodies, anywhere defining a struct
would be more ceremony than it is worth. Note the serialised key order —
alphabetical, not as written, because `Value`'s map is a `BTreeMap` unless
`preserve_order` is on.

### Working with `Value`

#### `Value`

The untyped JSON enum: `Null`, `Bool`, `Number`, `String`, `Array`, `Object`.

```
use serde_json::Value;

let v: Value = serde_json::from_str(r#"{"a":[1,"two",null]}"#).unwrap();

assert!(v.is_object());
assert!(v["a"].is_array());
assert!(v["a"][2].is_null());

match &v["a"][1] {
    Value::String(s) => assert_eq!(s, "two"),
    other => panic!("expected a string, got {other:?}"),
}
```

**When to use it:** whenever the shape isn't known at compile time. Match on it
when you need to handle every kind; use the `as_*` accessors when you only care
about one.

#### Indexing with `[]`

Infallible access by key or position, yielding `Null` when absent.

```
use serde_json::json;

let v = json!({ "user": { "name": "ada" }, "list": [10, 20] });

assert_eq!(v["user"]["name"], "ada");
assert_eq!(v["list"][1], 20);

// Missing keys and out-of-range indices are Null, not a panic.
assert!(v["missing"].is_null());
assert!(v["list"][99].is_null());
```

**When to use it:** quick reads where absent and null mean the same thing to
you. It cannot distinguish the two, and it panics if you index a non-container
by string — so `get` is safer for genuinely unknown documents.

#### `get` and the `as_*` accessors

Fallible access, returning `Option`.

```
use serde_json::json;

let v = json!({ "port": 8080, "debug": true, "name": "svc" });

assert_eq!(v.get("port").and_then(|p| p.as_u64()), Some(8080));
assert_eq!(v.get("debug").and_then(|d| d.as_bool()), Some(true));
assert_eq!(v.get("name").and_then(|n| n.as_str()), Some("svc"));

// Wrong type gives None rather than coercing.
assert_eq!(v.get("port").and_then(|p| p.as_str()), None);
assert_eq!(v.get("absent"), None);
```

**When to use it:** reading untyped data defensively. The `as_*` family never
coerces — `as_str` on a number is `None`, not `"8080"` — which is what makes a
chain of `and_then` a real validation rather than a hope.

#### `pointer`

Looks up a nested value by [JSON Pointer](https://datatracker.ietf.org/doc/html/rfc6901) path.

```
use serde_json::json;

let v = json!({ "a": { "b": [ { "c": 42 } ] } });

assert_eq!(v.pointer("/a/b/0/c").and_then(|v| v.as_i64()), Some(42));
assert_eq!(v.pointer("/a/x"), None);
assert_eq!(v.pointer("").unwrap(), &v); // <- empty path is the root
```

**When to use it:** reaching deep into a document by a path that is itself data —
a config key, a CLI argument, a rule in a table. Far clearer than chaining
`get`, and the path is a string you can store.

#### `Map`

The object type, with `preserve_order` deciding whether it keeps key order.

```
use serde_json::{Map, Value};

let mut map = Map::new();
map.insert("z".to_string(), Value::from(1));
map.insert("a".to_string(), Value::from(2));

let value = Value::Object(map);
assert_eq!(value["a"], 2);

// Without preserve_order, serialisation sorts keys.
assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"a":2,"z":1}"#);
```

**When to use it:** building objects programmatically, or as a
`#[serde(flatten)]` catch-all for unknown fields. The ordering caveat is the one
to remember: enable `preserve_order` if the output is a file someone diffs.

### Converting

#### `to_value` and `from_value`

Convert between your types and `Value` without going through text.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Point { x: i32, y: i32 }

let value = serde_json::to_value(Point { x: 1, y: 2 }).unwrap();
assert_eq!(value["x"], 1);

let back: Point = serde_json::from_value(value).unwrap();
assert_eq!(back, Point { x: 1, y: 2 });
```

**When to use it:** when part of a document is typed and part isn't — pull the
known section out of a `Value` into a struct, or build a `Value` from typed
pieces. It avoids the serialise-to-string-then-parse round trip people reach for
first, which is slower and can lose precision.

#### `Value::take`

Moves a value out, leaving `Null` behind.

```
use serde_json::json;

let mut v = json!({ "payload": { "big": [1, 2, 3] } });

let payload = v["payload"].take();

assert_eq!(payload["big"][0], 1);
assert!(v["payload"].is_null()); // <- left behind
```

**When to use it:** extracting a large sub-document you own from one you are
about to discard, without cloning it. The `Null` it leaves is the giveaway if
you use the original afterwards — so take it last, or `clone` if you still need
the whole thing.

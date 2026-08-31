---
title: "serde"
version: "1.0.229"
publisher: "David Tolnay (dtolnay), publish"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-08-31"
summary: "The serialization framework the whole ecosystem shares. Derive `Serialize` and `Deserialize` once, and your type reads and writes JSON, TOML, YAML, MessagePack and the rest without knowing about any of them."
categories: ["serialization", "derive", "no-std"]
repository: "https://github.com/serde-rs/serde"
---

## Overview

`serde` is a framework, not a format. It defines a data model in the middle —
maps, sequences, structs, enums, primitives — and two traits that connect a Rust
type to it:

- **`Serialize`** — how to take your type apart into that model.
- **`Deserialize`** — how to build it back up.

Formats sit on the other side. [`serde_json`](serde_json.md), `toml`,
`serde_yaml`, `rmp-serde`, `bincode` and dozens more implement the model, so a
type that derives these two traits works with all of them and knows about none:

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Config {
    host: String,
    port: u16,
}

let config = Config { host: "example.com".into(), port: 8080 };

// One derive, any number of formats.
let json = serde_json::to_string(&config).unwrap();
assert_eq!(json, r#"{"host":"example.com","port":8080}"#);

let back: Config = serde_json::from_str(&json).unwrap();
assert_eq!(back, config);
```

That indirection is the whole design, and it is why serde sits near the top of
the download charts: N types times M formats becomes N plus M.

**The derive is not enabled by default.** `serde = "1"` gives you the traits
alone; you want `features = ["derive"]`, and forgetting it produces a confusing
"cannot find derive macro" error. The crate is split three ways as of recent
releases — `serde_core` holds the traits, `serde_derive` is the proc macro, and
`serde` re-exports both — so a library needing only the traits can depend on
`serde_core` and keep the proc macro out of its dependents' build graphs.

**What the attributes are for.** The derive alone maps field names to keys
one-to-one, which is rarely what a real format wants. Almost everything you will
need to configure is a `#[serde(...)]` attribute: renaming to `camelCase`,
defaulting an absent field, skipping an empty one, choosing how an enum is
tagged. Those are documented here rather than in a format crate's docs, because
they belong to serde and apply to every format at once.

The costs are real but usually accepted. The derive pulls
[`syn`](syn.md), [`quote`](quote.md) and [`proc-macro2`](proc-macro2.md), and
compiling it is a noticeable share of a cold build in a project with many types.
Generated code is monomorphised per type and format, which trades binary size
for speed. `miniserde` and `nanoserde` exist for when that trade is wrong, at
the cost of the ecosystem — nothing else speaks their traits.

It requires Rust 1.56, and works `no_std` with `alloc`, or without `alloc` for
the subset that needs no allocation.

## When to use it

### Use case: Matching an external JSON convention

Rust fields are `snake_case`; a JSON API is usually `camelCase`. `rename_all`
translates the whole struct at once, so the Rust side stays idiomatic.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct User {
    first_name: String,
    last_name: String,
    #[serde(rename = "id")] // <- one field that doesn't follow the rule
    user_id: u64,
}

let json = r#"{"firstName":"Ada","lastName":"Lovelace","id":1}"#;
let user: User = serde_json::from_str(json).unwrap();

assert_eq!(user.first_name, "Ada");
assert_eq!(serde_json::to_string(&user).unwrap(), json);
```

**Why it fits:** the wire format is data, not something your code should have to
speak. Both directions use the same rule, so the round trip is exact — which is
what makes it safe to write the document back.

### Use case: Evolving a format without breaking old data

Config files and stored documents outlive the code that reads them. `default`
lets you add a field without invalidating everything written before it.

```
use serde::{Deserialize, Serialize};

fn default_retries() -> u32 { 3 }

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    endpoint: String,
    #[serde(default = "default_retries")]
    retries: u32,
    #[serde(default)] // <- Default::default(), so false
    verbose: bool,
}

// An old document, written before either field existed.
let old: Settings = serde_json::from_str(r#"{"endpoint":"https://api"}"#).unwrap();
assert_eq!(old.retries, 3);
assert!(!old.verbose);

// A new one, with them present.
let new: Settings = serde_json::from_str(
    r#"{"endpoint":"https://api","retries":5,"verbose":true}"#,
).unwrap();
assert_eq!(new.retries, 5);
```

**Why it fits:** adding a field is no longer a breaking change to the format.
Note the pairing: `#[serde(deny_unknown_fields)]` is the opposite policy, for
when a typo in a config should be an error rather than silently ignored — you
usually want one or the other deliberately, not neither.

### Use case: Modelling a tagged union

An enum is the natural type for "one of several message kinds", and serde offers
four ways to put that on the wire. The internally tagged form is what most JSON
APIs use.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
}

let json = serde_json::to_string(&Event::Click { x: 1, y: 2 }).unwrap();
assert_eq!(json, r#"{"type":"click","x":1,"y":2}"#);

let parsed: Event = serde_json::from_str(r#"{"type":"key_press","key":"a"}"#).unwrap();
assert!(matches!(parsed, Event::KeyPress { key } if key == "a"));
```

**Why it fits:** the discriminant is a field like any other, which is how these
documents are usually specified. The alternative representations are covered
below; picking the wrong one is the most common reason an enum won't round-trip
against someone else's API.

## API map

Serde's surface is two traits and a set of attributes. In practice you write the
derive and then reach for attributes, so those are what this map covers,
grouped by where they go. Everything below is inside `#[serde(...)]`.

### The traits

#### `Serialize` and `Deserialize`

The two traits. Deriving both is the normal case; derive only what you need.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Outgoing { id: u64 }

#[derive(Deserialize)]
struct Incoming { ok: bool }

assert_eq!(serde_json::to_string(&Outgoing { id: 1 }).unwrap(), r#"{"id":1}"#);
let got: Incoming = serde_json::from_str(r#"{"ok":true}"#).unwrap();
assert!(got.ok);
```

**When to use it:** on any type crossing a boundary. Deriving both when you only
send or only receive costs compile time and can force bounds you don't need — a
response type usually needs `Deserialize` alone.

#### Implementing them by hand

The derive covers nearly everything, but the traits are public.

```
use serde::{Serialize, Serializer};

struct Celsius(f64);

impl Serialize for Celsius {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Emit as a string with a unit, rather than a bare number.
        serializer.serialize_str(&format!("{}C", self.0))
    }
}

assert_eq!(serde_json::to_string(&Celsius(21.5)).unwrap(), r#""21.5C""#);
```

**When to use it:** rarely — for a type whose representation isn't a
rearrangement of its fields. Prefer `serialize_with` on a field, or `from`/`into`
on the container, both of which get you the same result without hand-writing a
trait impl.

### Container attributes

#### `rename_all`

Applies a naming convention to every field or variant.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Env {
    database_url: String,
}

let value = Env { database_url: "postgres://".into() };
assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"DATABASE_URL":"postgres://"}"#);
```

**When to use it:** matching an external convention in one line rather than
renaming each field. It takes `camelCase`, `PascalCase`, `snake_case`,
`SCREAMING_SNAKE_CASE`, `kebab-case` and more; `rename_all_fields` applies to an
enum's *variant* fields rather than the variant names.

#### `deny_unknown_fields`

Rejects input containing keys the struct doesn't have.

```
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Strict { name: String }

assert!(serde_json::from_str::<Strict>(r#"{"name":"a"}"#).is_ok());

// A typo is now an error instead of being ignored.
let err = serde_json::from_str::<Strict>(r#"{"name":"a","nmae":"b"}"#).unwrap_err();
assert!(err.to_string().contains("unknown field"));
```

**When to use it:** config files, where a misspelled key silently doing nothing
is a bad afternoon. Avoid it for API responses you don't control — a server
adding a field would break your client.

#### `default` on a container

Fills every missing field from `Default`, rather than naming them one by one.

```
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(default)]
struct Options {
    verbose: bool,
    retries: u32,
}

let empty: Options = serde_json::from_str("{}").unwrap();
assert_eq!(empty, Options::default());

let partial: Options = serde_json::from_str(r#"{"retries":2}"#).unwrap();
assert_eq!(partial, Options { verbose: false, retries: 2 });
```

**When to use it:** options structs where every field is optional. It requires
`Default` on the type, and is the concise alternative to `#[serde(default)]` on
each field.

#### `transparent`

Serialises a one-field wrapper as the field itself.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct UserId(u64);

assert_eq!(serde_json::to_string(&UserId(7)).unwrap(), "7"); // <- not [7] or {"0":7}
let id: UserId = serde_json::from_str("7").unwrap();
assert_eq!(id.0, 7);
```

**When to use it:** newtypes that exist for type safety and shouldn't appear in
the format. Without it a tuple struct serialises as its inner value anyway in
JSON, but `transparent` makes the intent explicit and holds across formats that
would otherwise wrap it.

#### `from`, `into` and `try_from`

Converts through another type on the way in or out.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[derive(Clone)]
struct Version { major: u32, minor: u32 }

impl From<String> for Version {
    fn from(s: String) -> Self {
        let (a, b) = s.split_once('.').unwrap_or(("0", "0"));
        Version { major: a.parse().unwrap_or(0), minor: b.parse().unwrap_or(0) }
    }
}

impl From<Version> for String {
    fn from(v: Version) -> String { format!("{}.{}", v.major, v.minor) }
}

let v: Version = serde_json::from_str(r#""2.7""#).unwrap();
assert_eq!((v.major, v.minor), (2, 7));
assert_eq!(serde_json::to_string(&v).unwrap(), r#""2.7""#);
```

**When to use it:** when the stored form is a different shape entirely — a
struct stored as a string, a validated type stored as its raw input. `try_from`
is the fallible version, and is how you validate during deserialisation rather
than after it.

### Field attributes

#### `rename` and `alias`

Renames one field; `alias` accepts extra names when reading.

```
use serde::Deserialize;

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "ID", alias = "id", alias = "identifier")]
    id: u64,
}

for text in [r#"{"ID":1}"#, r#"{"id":1}"#, r#"{"identifier":1}"#] {
    let r: Record = serde_json::from_str(text).unwrap();
    assert_eq!(r.id, 1);
}
```

**When to use it:** `rename` for a field whose wire name differs from the Rust
one; `alias` for accepting an old name after a format change, which lets you
migrate without breaking existing documents. `alias` affects reading only.

#### `default` on a field

Supplies a value when the field is absent.

```
use serde::Deserialize;

fn one() -> u32 { 1 }

#[derive(Deserialize)]
struct Job {
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "one")]
    parallelism: u32,
}

let job: Job = serde_json::from_str(r#"{"name":"build"}"#).unwrap();
assert!(job.tags.is_empty());
assert_eq!(job.parallelism, 1);
```

**When to use it:** any optional field. Bare `default` uses `Default::default()`;
the string form names a function, which is how you get a non-zero default.
`Option<T>` is the other approach, and says something different — absent versus
present-and-null are distinguishable with `Option`.

#### `skip_serializing_if`

Omits a field from the output when a predicate says so.

```
use serde::Serialize;

#[derive(Serialize)]
struct Payload {
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

let value = Payload { id: 1, note: None, tags: vec![] };
assert_eq!(serde_json::to_string(&value).unwrap(), r#"{"id":1}"#);
```

**When to use it:** keeping documents small and diffs clean by leaving out
nothing-values. The function takes a reference to the field and returns `bool`,
so any predicate works. Pair it with `default` so the omitted field reads back.

#### `skip`

Leaves a field out of both directions entirely.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct Session {
    user: String,
    #[serde(skip)]
    cache: Vec<u8>, // <- never written, never read
}

let s = Session { user: "ada".into(), cache: vec![1, 2, 3] };
assert_eq!(serde_json::to_string(&s).unwrap(), r#"{"user":"ada"}"#);

let back: Session = serde_json::from_str(r#"{"user":"ada"}"#).unwrap();
assert!(back.cache.is_empty()); // <- filled from Default
```

**When to use it:** derived state, caches, handles — anything meaningless
outside the running process. The field must implement `Default` for
deserialisation, since something has to go there.

#### `flatten`

Inlines a nested struct's fields into the parent, or collects the leftovers.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Page {
    items: Vec<u32>,
    #[serde(flatten)]
    paging: Paging,
}

#[derive(Serialize, Deserialize)]
struct Paging { offset: u32, limit: u32 }

let value = Page { items: vec![1], paging: Paging { offset: 0, limit: 10 } };
let json = serde_json::to_string(&value).unwrap();

// One flat object, not a nested "paging" key.
assert_eq!(json, r#"{"items":[1],"offset":0,"limit":10}"#);
```

**When to use it:** sharing a common group of fields across types, or capturing
unknown keys into a `HashMap`. It is incompatible with `deny_unknown_fields`,
and it forces a buffering deserialiser, so it costs a little performance on hot
paths.

#### `serialize_with` and `with`

Uses your own functions for one field's representation.

```
use serde::{Deserialize, Serialize};

mod millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_millis() as u64)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        Ok(Duration::from_millis(u64::deserialize(d)?))
    }
}

#[derive(Serialize, Deserialize)]
struct Timeout {
    #[serde(with = "millis")]
    after: std::time::Duration,
}

let t = Timeout { after: std::time::Duration::from_secs(2) };
assert_eq!(serde_json::to_string(&t).unwrap(), r#"{"after":2000}"#);
```

**When to use it:** representing a foreign type the way your format expects — a
timestamp as an integer, bytes as base64. `with` names a module containing both
functions; `serialize_with` and `deserialize_with` name them individually when
you need only one.

### Enum representations

#### Externally tagged (the default)

The variant name wraps its content.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Message {
    Text { body: String },
    Ping,
}

let json = serde_json::to_string(&Message::Text { body: "hi".into() }).unwrap();
assert_eq!(json, r#"{"Text":{"body":"hi"}}"#);
assert_eq!(serde_json::to_string(&Message::Ping).unwrap(), r#""Ping""#);
```

**When to use it:** when you own both ends and want the representation that is
always unambiguous. It is the default because it round-trips for every enum
shape, including the ones the other three cannot handle.

#### Internally tagged — `tag`

The discriminant becomes a field alongside the content.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Shape {
    Circle { r: f64 },
    Square { side: f64 },
}

let json = serde_json::to_string(&Shape::Circle { r: 1.0 }).unwrap();
assert_eq!(json, r#"{"kind":"Circle","r":1.0}"#);
```

**When to use it:** most JSON APIs, which specify exactly this shape. It only
works for struct variants and newtype variants wrapping a struct — a tuple
variant has nowhere to put the tag, and the derive rejects it at compile time.

#### Adjacently tagged — `tag` and `content`

Tag and content sit in two named fields.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Value {
    Int(i64),
    Pair(i64, i64),
}

assert_eq!(serde_json::to_string(&Value::Int(1)).unwrap(), r#"{"t":"Int","c":1}"#);
assert_eq!(serde_json::to_string(&Value::Pair(1, 2)).unwrap(), r#"{"t":"Pair","c":[1,2]}"#);
```

**When to use it:** when you want a tag but have tuple or primitive variants that
internal tagging can't express. It handles every enum shape, at the cost of an
extra level of nesting.

#### `untagged`

No discriminant; variants are tried in order until one fits.

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Id {
    Number(u64),
    Text(String),
}

let n: Id = serde_json::from_str("7").unwrap();
assert!(matches!(n, Id::Number(7)));

let t: Id = serde_json::from_str(r#""abc""#).unwrap();
assert!(matches!(t, Id::Text(ref s) if s == "abc"));
```

**When to use it:** a field that legitimately accepts more than one shape — an
id that may be a number or a string. Order matters, because the first variant
that parses wins, and error messages are poor when nothing matches: serde can
only say that no variant fitted, not why each failed.

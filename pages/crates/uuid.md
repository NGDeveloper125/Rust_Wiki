---
title: "uuid"
version: "1.29.2"
publisher: "Ashley Mannix (KodrAus), rust-lang-owner"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-11"
summary: "128-bit identifiers: parsing and formatting with no dependencies at all, and generation opt-in one version at a time — random v4, time-ordered v7, or deterministic v5 derived from a name."
domain: "Randomness & IDs"
categories: ["data-structures", "no-std", "parser-implementations"]
repository: "https://github.com/uuid-rs/uuid"
docs: "https://docs.rs/uuid/1.29.2/uuid/"
---

## Overview

A UUID is 128 bits you can mint without asking anyone's permission. No central
allocator, no coordination, no round trip to a sequence in the database — two
machines that have never met will not collide, and that is the entire value
proposition. `uuid` is the crate that makes them in Rust: generate, parse,
format, and inspect.

The thing to understand before adding it is that **the crate is almost entirely
opt-in**, and the default does not generate anything. `default = ["std"]`, which
gives you parsing, formatting and inspection with *zero* dependencies — `uuid`
on its own. Every generator sits behind its own feature, and that is also where
the dependency cost lives. Measured on 1.29.2 with `cargo tree`:

- parsing and formatting only — **no dependencies**
- `v4` or `v7` — two crates (`getrandom`, `cfg-if`)
- `v5` — one crate (`sha1_smol`, a single self-contained hash implementation)
- `v3` — **eight** crates, because `md-5` drags in the whole `digest` stack
- `v1` or `v6` — two crates (`atomic`, `bytemuck`)
- all seven generators at once — twelve crates
- `fast-rng`, which swaps `getrandom` for [`rand`](rand.md) — six

So `uuid = { version = "1", features = ["v4"] }` is a genuinely cheap
dependency, and the common mistake is enabling more than you use. If you only
need to read UUIDs that something else produced, enable nothing.

The real decision is **which version to generate**, and it is not a style
question:

- **v4** — 122 random bits. The default answer. Unguessable, uncorrelated, and
  it tells an observer nothing.
- **v7** — a Unix millisecond timestamp in the high bits, random below. Sorts
  chronologically as bytes, which is what you want for a database key. Leaks
  creation time.
- **v5** — SHA-1 of a namespace plus a name. Deterministic: the same inputs
  always produce the same UUID, which makes imports idempotent. Not secret, and
  not a hash you should treat as one.
- **v3** — the same idea with MD5. Exists for compatibility; prefer v5.
- **v1 / v6** — timestamp plus a node ID, historically the machine's MAC
  address. v6 is v1 with the timestamp fields reordered so they sort. Mostly of
  interest when something else hands you one.
- **v8** — 122 bits of whatever you want, with the version and variant markers
  applied for you.

`no_std` is real but partial. Parsing, formatting, v3, v5 and v8 build for
`thumbv7m-none-eabi` with `default-features = false`; `v4` and `v7` do not,
because `getrandom` has no backend on bare metal. On
`wasm32-unknown-unknown` the crate stops the build itself with a message naming
the three features that fix it.

One constraint that will decide this for some projects: **1.29.2 declares
`rust-version = "1.89.0"`**. That is a recent compiler for a crate this far into
a 1.x line, so a project pinned to an older toolchain needs an older `uuid`.

If a UUID is not actually what you want: `ulid` gives you the same sortable
time-prefixed idea in a 26-character string, `nanoid` gives short URL-safe IDs,
and if you just need unpredictable bytes then [`rand`](rand.md) or
[`fastrand`](fastrand.md) is a smaller ask than an identifier format.

## When to use it

### Use case: An identifier for a record you are about to create

The row does not exist yet and you need its ID now — to put in a log line, to
return to the caller, to reference from a sibling row you are inserting in the
same transaction. A database sequence cannot tell you the value until the insert
lands; a v4 can.

```
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Account {
    id: Uuid,
    email: String,
}

fn main() {
    let account = Account {
        id: Uuid::new_v4(),
        email: "ada@example.org".into(),
    };

    // The id exists before anything is persisted, so it can be logged,
    // returned, and referenced by other rows in the same transaction.
    assert_eq!(account.id.get_version_num(), 4);
    assert!(!account.id.is_nil());

    // serde writes it as the hyphenated string in any human-readable format.
    let json = serde_json::to_string(&account).unwrap();
    let expected = format!(r#"{{"id":"{}","email":"ada@example.org"}}"#, account.id);
    assert_eq!(json, expected);

    let back: Account = serde_json::from_str(&json).unwrap();
    assert_eq!(back, account);
}
```

**Why it fits:** two features — `v4` and `serde` — and three crates in the
dependency tree buy you an identifier that needs no coordination with anything.
The ID is available at the moment you decide the record exists rather than at
the moment the database agrees, which is what makes it possible to build a whole
object graph in memory and insert it in one go.

### Use case: A primary key that does not scatter writes across the index

v4 is random, so consecutive inserts land in random places in a B-tree. Every
insert dirties a different page, the cache hit rate collapses, and the index
fragments. v7 fixes this by putting the timestamp in the high bits, so new keys
sort to the end and inserts stay sequential.

```
use uuid::Uuid;

fn main() {
    // now_v7 is documented to be ordered by creation for the whole process,
    // not merely to millisecond granularity.
    let ids: Vec<Uuid> = (0..1000).map(|_| Uuid::now_v7()).collect();

    let inversions = ids.windows(2).filter(|w| w[0] >= w[1]).count();
    assert_eq!(inversions, 0); // <- every one sorts after the one before it

    // And they really are being minted inside the same millisecond, so the
    // ordering is coming from the counter rather than from the clock.
    let same_ms = ids
        .windows(2)
        .filter(|w| w[0].get_timestamp().unwrap().to_unix() == w[1].get_timestamp().unwrap().to_unix())
        .count();
    assert!(same_ms > 900, "{same_ms} of 999 pairs shared a millisecond");

    // Sorting the batch is a no-op: it is already in insertion order.
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(sorted, ids);

    // v4 gives up on all of that, by design.
    let v4s: Vec<Uuid> = (0..1000).map(|_| Uuid::new_v4()).collect();
    let v4_inversions = v4s.windows(2).filter(|w| w[0] >= w[1]).count();
    assert!(v4_inversions > 300, "{v4_inversions} of 999 pairs out of order");
}
```

The cost is that a v7 publishes its creation time to anyone holding it. For a
primary key that is usually fine. For a password reset token it is not — use v4,
or something built for secrets.

**Why it fits:** the sortability is a property of the bytes, so it survives
everything — the database's index, a `BTreeMap`, a sorted file, a comparison in
a different language. Nothing has to know the IDs are UUIDs, and the ordering
guarantee `now_v7` documents is per-process, which is exactly the scope that
matters for a run of inserts from one service.

### Use case: A stable ID derived from a name, so re-running an import is a no-op

You are importing records from an upstream system that has its own identifiers.
You want your own UUID for each, you want the same UUID every time, and you do
not want to keep a mapping table. v5 hashes a namespace and a name into a UUID,
so the ID is a pure function of the input.

```
use uuid::Uuid;

fn main() {
    // The namespace says what kind of name this is. The four RFC namespaces
    // are provided as constants.
    let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"example.org");
    assert_eq!(id.to_string(), "aad03681-8b63-5304-89e0-8ca8f49461b5");

    // Deterministic: re-running the import produces the same key, so the
    // insert can be an upsert and there is no mapping table to maintain.
    assert_eq!(id, Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"example.org"));

    // A different namespace over the same name is a different ID, which is
    // how you keep two kinds of name from colliding.
    assert_ne!(id, Uuid::new_v5(&Uuid::NAMESPACE_URL, b"example.org"));

    // Namespaces nest: derive one for your tenant, then names within it.
    let tenant = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"tenant");
    let user = Uuid::new_v5(&tenant, b"user-42");
    assert_eq!(user.to_string(), "96e05871-27d0-508c-8f5d-b1fdc46e8cd0");
    assert_eq!(user.get_version_num(), 5);
}
```

What it is not: a secret. The namespaces are public constants and SHA-1 is fast,
so anyone who can guess the name can compute the ID. Derive IDs this way, never
tokens.

**Why it fits:** idempotence without state. The expensive part of an importer is
usually the bookkeeping that stops it double-inserting on a retry; a
deterministic key deletes that problem, because the second run computes the same
primary key and the database's own uniqueness constraint does the rest. `v5`
costs one extra crate, and no randomness, so it also works on bare metal.

## API map

Everything below is an inherent method on `Uuid` unless it says otherwise.

### Generating one

#### `Uuid::new_v4`

122 random bits, with the version and variant markers set. Needs the `v4`
feature, which pulls in `getrandom`.

```
use uuid::Uuid;

fn main() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    assert_ne!(a, b);
    assert_eq!(a.get_version_num(), 4);
    assert_eq!(a.get_variant(), uuid::Variant::RFC4122);

    // Nothing recoverable is encoded in it -- that is the point.
    assert_eq!(a.get_timestamp(), None);
    assert_eq!(a.get_node_id(), None);
}
```

**When to use it:** the default choice, and the right one unless you have a
specific reason to want otherwise. Reach for `now_v7` instead when the IDs are
database keys and insert locality matters; stay on v4 when the ID will be
visible to someone who should not learn when it was created.

#### `Uuid::now_v7`

A v7 from the current system time. The documented guarantee is stronger than the
timestamp alone would give: every UUID from this method *in the same process* is
ordered by creation, because it draws on a shared counter.

```
use uuid::Uuid;

fn main() {
    let first = Uuid::now_v7();
    let second = Uuid::now_v7();

    assert!(first < second);
    assert_eq!(first.get_version_num(), 7);

    // The timestamp is recoverable, to the millisecond and no finer.
    let (secs, nanos) = first.get_timestamp().unwrap().to_unix();
    assert!(secs > 1_700_000_000);
    assert_eq!(nanos % 1_000_000, 0); // <- whole milliseconds only
}
```

**When to use it:** whenever a v7 is what you want and the clock you want is the
system clock, which is nearly always. Use `new_v7` only when the timestamp has
to come from somewhere else.

#### `Uuid::new_v7` and `ContextV7`

`new_v7` takes an explicit `Timestamp`. The counter inside that `Timestamp` is
what provides ordering within a millisecond, so **which context you build it
with decides whether the result sorts at all** — and `NoContext` means no
counter.

```
use uuid::{ContextV7, NoContext, Timestamp, Uuid};

fn main() {
    // NoContext: the sub-millisecond bits are pure randomness, so two ids from
    // the same millisecond land in random order.
    let ts = Timestamp::from_unix(NoContext, 1_700_000_000, 0);
    let ascending = (0..1000).filter(|_| Uuid::new_v7(ts) < Uuid::new_v7(ts)).count();
    assert!(
        (300..700).contains(&ascending),
        "{ascending} of 1000 pairs ascending -- that is a coin flip"
    );

    // ContextV7 carries the counter, and the order is restored.
    let ctx = ContextV7::new();
    let ascending = (0..1000)
        .filter(|_| {
            let a = Uuid::new_v7(Timestamp::from_unix(&ctx, 1_700_000_000, 0));
            let b = Uuid::new_v7(Timestamp::from_unix(&ctx, 1_700_000_000, 0));
            a < b
        })
        .count();
    assert_eq!(ascending, 1000);
}
```

**When to use it:** when the timestamp comes from the data rather than the clock
— backfilling IDs for records that already have a `created_at`, or generating
them in a test with a fixed time. Build the context once and share it; a fresh
`ContextV7` per call is the same bug as `NoContext`.

#### `Uuid::new_v5` and `Uuid::new_v3`

Deterministic UUIDs: SHA-1 (v5) or MD5 (v3) over a namespace UUID and a byte
string. Same inputs, same output, forever.

```
use uuid::Uuid;

fn main() {
    let name = b"example.org";

    assert_eq!(
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, name).to_string(),
        "aad03681-8b63-5304-89e0-8ca8f49461b5"
    );
    assert_eq!(
        Uuid::new_v3(&Uuid::NAMESPACE_DNS, name).to_string(),
        "04738bdf-b25a-3829-a801-b21a1d25095b"
    );

    // The four namespaces from the RFC. Note X500 is ...b814, not ...b813.
    assert_eq!(Uuid::NAMESPACE_DNS.to_string(),  "6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    assert_eq!(Uuid::NAMESPACE_URL.to_string(),  "6ba7b811-9dad-11d1-80b4-00c04fd430c8");
    assert_eq!(Uuid::NAMESPACE_OID.to_string(),  "6ba7b812-9dad-11d1-80b4-00c04fd430c8");
    assert_eq!(Uuid::NAMESPACE_X500.to_string(), "6ba7b814-9dad-11d1-80b4-00c04fd430c8");
}
```

**When to use it:** `new_v5` whenever you want an ID that is a function of
something you already have — an upstream key, a URL, a file path. Use `new_v3`
only to match IDs that something else already generated with MD5: it is the
weaker hash and it costs eight dependencies against v5's one.

#### `Uuid::now_v1` and `Uuid::now_v6`

Timestamp plus a six-byte node ID. v6 is the same data with the timestamp
fields reordered into most-significant-first order, and that reordering is the
whole point: **v1 does not sort chronologically and v6 does.**

```
use uuid::{NoContext, Timestamp, Uuid};

fn main() {
    let node = [0u8; 6];
    let base = 1_700_000_000;

    // Seven and eight minutes past the same instant.
    let t7 = Timestamp::from_unix(NoContext, base + 420, 0);
    let t8 = Timestamp::from_unix(NoContext, base + 480, 0);

    let v1_earlier = Uuid::new_v1(t7, &node);
    let v1_later = Uuid::new_v1(t8, &node);
    assert_eq!(v1_earlier.to_string(), "ff06aa00-833b-11ee-8000-000000000000");
    assert_eq!(v1_later.to_string(),   "22c9f000-833c-11ee-8000-000000000000");
    assert!(v1_earlier > v1_later); // <- the earlier id sorts LAST

    // The same two instants as v6.
    assert_eq!(Uuid::new_v6(t7, &node).to_string(), "1ee833bf-f06a-6a00-8000-000000000000");
    assert_eq!(Uuid::new_v6(t8, &node).to_string(), "1ee833c2-2c9f-6000-8000-000000000000");
    assert!(Uuid::new_v6(t7, &node) < Uuid::new_v6(t8, &node));

    // The low field holds 100ns ticks in 32 bits, so it wraps every ~429
    // seconds -- a v1 sequence inverts roughly every seven minutes.
    let stamps: Vec<Timestamp> = (0..500)
        .map(|i| Timestamp::from_unix(NoContext, base + i * 60, 0))
        .collect();
    let v1: Vec<Uuid> = stamps.iter().map(|t| Uuid::new_v1(*t, &node)).collect();
    let v6: Vec<Uuid> = stamps.iter().map(|t| Uuid::new_v6(*t, &node)).collect();
    assert_eq!(v1.windows(2).filter(|w| w[0] >= w[1]).count(), 69);
    assert_eq!(v6.windows(2).filter(|w| w[0] >= w[1]).count(), 0);
}
```

**When to use it:** mostly, don't — generate v7 instead, which gets you
sortability without broadcasting a hardware address. Reach for these when a
protocol or an existing table demands a v1 or v6, and note that the node ID is
yours to supply: the crate will not go looking for your MAC address, so passing
random bytes is both allowed and usually wiser.

#### `Uuid::new_v8`

Takes sixteen bytes and returns them as a UUID with the version and variant
markers forced in. It is a `const fn`, and it is the sanctioned way to put your
own data in a UUID-shaped field.

```
use uuid::Uuid;

fn main() {
    let input = [0xffu8; 16];
    let id = Uuid::new_v8(input);

    assert_eq!(id.to_string(), "ffffffff-ffff-8fff-bfff-ffffffffffff");

    // Exactly two bytes are rewritten -- six bits in total, so 122 of your
    // 128 survive. Anything you encode has to dodge those six.
    let changed: Vec<usize> = (0..16).filter(|&i| id.as_bytes()[i] != input[i]).collect();
    assert_eq!(changed, vec![6, 8]);

    assert_eq!(Uuid::new_v8([0x00; 16]).to_string(), "00000000-0000-8000-8000-000000000000");
}
```

**When to use it:** when you have to carry structured data through something that
only accepts a UUID column, and you would rather it be a documented v8 than a
v4 with secrets in it. If you control both ends, a real field is better.

#### `uuid!`

Parses a literal at compile time into a `const Uuid`. A bad literal fails the
build.

```
use uuid::{uuid, Uuid};

const TENANT: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

fn main() {
    assert_eq!(TENANT.to_string(), "67e55044-10b1-426f-9247-bb680e5fe0c8");
    assert_eq!(TENANT.get_version_num(), 4);
}
```

A malformed literal is caught by the compiler rather than at runtime:

```
// no-compile
use uuid::{uuid, Uuid};

// error[E0080]: evaluation panicked: invalid UUID
const BAD: Uuid = uuid!("not-a-uuid");

fn main() {
    println!("{BAD}");
}
```

**When to use it:** for every UUID that appears literally in your source — fixed
namespaces, well-known tenant IDs, test fixtures. It needs no features at all,
and it replaces `parse_str(...).unwrap()` with a build error. The
`macro-diagnostics` feature makes the failure message point at the literal
itself, at the cost of a proc-macro dependency.

### Reading one from text

#### `Uuid::try_parse`

The `const fn` parser. It accepts four shapes — plain hyphenated, 32 bare hex
digits, braced, and the `urn:uuid:` form — in either case. It accepts nothing
else.

```
use uuid::Uuid;

fn main() {
    let expected = "67e55044-10b1-426f-9247-bb680e5fe0c8";

    for input in [
        "67e55044-10b1-426f-9247-bb680e5fe0c8",
        "67e5504410b1426f9247bb680e5fe0c8",
        "{67e55044-10b1-426f-9247-bb680e5fe0c8}",
        "urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8",
        "67E55044-10B1-426F-9247-BB680E5FE0C8",
    ] {
        assert_eq!(Uuid::try_parse(input).unwrap().to_string(), expected, "{input}");
    }

    // The prefixes do not combine with the unhyphenated form, and there is no
    // trimming -- a trailing newline off a file or a socket is a parse error.
    for input in [
        "{67e5504410b1426f9247bb680e5fe0c8}",
        "urn:uuid:67e5504410b1426f9247bb680e5fe0c8",
        " 67e55044-10b1-426f-9247-bb680e5fe0c8",
        "67e55044-10b1-426f-9247-bb680e5fe0c8\n",
        "{67e55044-10b1-426f-9247-bb680e5fe0c8",
    ] {
        assert!(Uuid::try_parse(input).is_err(), "{input:?} should not parse");
    }
}
```

**When to use it:** as the default parser, and the only one that works in a
`const` item. `FromStr` is implemented too, so `s.parse::<Uuid>()` goes through
the same grammar. Trim your input first — the crate will not do it for you.

#### `Uuid::parse_str`

The same grammar, but the error says what went wrong. Being a `const fn` costs
`try_parse` its error detail: every failure collapses into one opaque variant.

```
use uuid::Uuid;

fn main() {
    let short = "67e55044-10b1-426f-9247-bb680e5fe0c";

    // try_parse: one message for every possible mistake.
    assert_eq!(
        Uuid::try_parse(short).unwrap_err().to_string(),
        "failed to parse a UUID"
    );

    // parse_str: the group, the expected length and the length found.
    assert_eq!(
        Uuid::parse_str(short).unwrap_err().to_string(),
        "invalid group length in group 4: expected 12, found 11"
    );
    assert_eq!(
        Uuid::parse_str("").unwrap_err().to_string(),
        "invalid length: found 0"
    );
    assert_eq!(
        Uuid::parse_str("67e550-4410b1-426f-9247-bb680e5fe0c8").unwrap_err().to_string(),
        "invalid group length in group 0: expected 8, found 6"
    );
}
```

**When to use it:** when a human will read the error — a config file, a CLI
argument, an API request you are rejecting with a message. The docs say
`parse_str` will eventually be deprecated in favour of `try_parse`, so do not
build on it where the detail does not matter. `try_parse_ascii` is the same
parser over `&[u8]`, for input you have not validated as UTF-8.

### Writing one as text

#### `hyphenated`, `simple`, `braced`, `urn`

`Display` gives the lowercase hyphenated form. The four adapters give the other
representations; each has an `as_*` twin that borrows instead of copying.

```
use uuid::Uuid;

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    assert_eq!(id.to_string(),          "67e55044-10b1-426f-9247-bb680e5fe0c8");
    assert_eq!(id.hyphenated().to_string(), "67e55044-10b1-426f-9247-bb680e5fe0c8");
    assert_eq!(id.simple().to_string(),  "67e5504410b1426f9247bb680e5fe0c8");
    assert_eq!(id.braced().to_string(),  "{67e55044-10b1-426f-9247-bb680e5fe0c8}");
    assert_eq!(id.urn().to_string(),     "urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8");

    // Uppercase is a format flag, not a separate adapter.
    assert_eq!(format!("{id:X}"), "67E55044-10B1-426F-9247-BB680E5FE0C8");

    // Debug prints the value, not the struct.
    assert_eq!(format!("{id:?}"), "67e55044-10b1-426f-9247-bb680e5fe0c8");
}
```

**When to use it:** `simple` where the hyphens are wasted bytes — URL path
segments, filenames, cache keys. `braced` and `urn` to match a format something
else expects. Otherwise leave it on `Display`, which is the form every other
system will recognise.

#### `encode_lower`, `encode_upper` and `encode_buffer`

Write the text into a caller-supplied buffer and return a `&mut str`. No
allocation, which is what makes formatting work in `no_std`.

```
use uuid::Uuid;

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    // encode_buffer is a const fn returning a [u8; 45] -- big enough for the
    // longest form, the urn.
    assert_eq!(Uuid::encode_buffer().len(), 45);

    let mut buf = Uuid::encode_buffer();
    let s: &mut str = id.hyphenated().encode_lower(&mut buf);
    assert_eq!(s, "67e55044-10b1-426f-9247-bb680e5fe0c8");

    let mut buf = Uuid::encode_buffer();
    assert_eq!(
        id.simple().encode_upper(&mut buf),
        "67E5504410B1426F9247BB680E5FE0C8"
    );
}
```

**When to use it:** in a hot loop, or anywhere without an allocator. Elsewhere
`to_string()` is clearer and the allocation will not be what slows you down.

### Bytes and integers

#### `as_bytes`, `from_bytes`, `from_bytes_ref` and `from_slice`

A `Uuid` *is* its sixteen bytes, in RFC 9562 network order. The type is a
`[u8; 16]` underneath — alignment 1, not 16 — so these conversions are free.

```
use uuid::Uuid;

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    assert_eq!(
        id.as_bytes(),
        &[0x67, 0xe5, 0x50, 0x44, 0x10, 0xb1, 0x42, 0x6f,
          0x92, 0x47, 0xbb, 0x68, 0x0e, 0x5f, 0xe0, 0xc8]
    );
    assert_eq!(Uuid::from_bytes(*id.as_bytes()), id);
    assert_eq!(std::mem::size_of::<Uuid>(), 16);
    assert_eq!(std::mem::align_of::<Uuid>(), 1); // <- a byte array, not a u128

    // Borrow a UUID out of bytes you already have, with no copy.
    let raw: [u8; 16] = *id.as_bytes();
    let borrowed: &Uuid = Uuid::from_bytes_ref(&raw);
    assert_eq!(borrowed, &id);

    // from_slice is the checked version, and its error is specific.
    assert_eq!(
        Uuid::from_slice(&[0u8; 15]).unwrap_err().to_string(),
        "invalid length: expected 16 bytes, found 15"
    );
    assert!(Uuid::from_slice(&[0u8; 16]).is_ok());
}
```

**When to use it:** `from_bytes` for a fixed-size array you already trust,
`from_slice` for sixteen bytes off a wire or out of a `BLOB` column where the
length is not proven. `from_bytes_ref` when you are scanning a packed buffer and
do not want to copy each one out.

#### `from_bytes_le` and `to_bytes_le` — the GUID trap

Microsoft's GUID stores the first three fields little-endian and the last two
big-endian. Reading those bytes with `from_bytes` instead of `from_bytes_le`
produces a valid, wrong UUID — no error, no panic, just a different identifier.

```
use uuid::Uuid;

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    // Only the first eight bytes move; the last eight are already big-endian.
    assert_eq!(
        id.to_bytes_le(),
        [0x44, 0x50, 0xe5, 0x67, 0xb1, 0x10, 0x6f, 0x42,
         0x92, 0x47, 0xbb, 0x68, 0x0e, 0x5f, 0xe0, 0xc8]
    );

    // The same sixteen bytes read the two ways are two different UUIDs, and
    // nothing tells you which one you meant.
    let bytes = *id.as_bytes();
    assert_eq!(Uuid::from_bytes(bytes), id);
    assert_eq!(
        Uuid::from_bytes_le(bytes).to_string(),
        "4450e567-b110-6f42-9247-bb680e5fe0c8"
    );

    // Round trip correctly and it is a no-op.
    assert_eq!(Uuid::from_bytes_le(id.to_bytes_le()), id);

    // The field accessors expose the same split. `as_fields` is network
    // order; `to_fields_le` swaps the first three.
    assert_eq!(id.as_fields().0, 0x67e5_5044);
    assert_eq!(id.to_fields_le().0, 0x4450_e567);
    let (d1, d2, d3, d4) = id.as_fields();
    assert_eq!(Uuid::from_fields(d1, d2, d3, d4), id);
}
```

**When to use it:** whenever the bytes came from or are going to a Microsoft
stack — a SQL Server `uniqueidentifier`, a .NET `Guid.ToByteArray()`, a Windows
API struct. Everywhere else use the plain versions. If IDs from one system
mysteriously do not match the same IDs from another, this is the first thing to
check.

#### `as_u128` and `from_u128`

The whole value as a single integer, big-endian. `to_u128_le` and `from_u128_le`
are the byte-swapped pair.

```
use uuid::Uuid;

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    assert_eq!(id.as_u128(), 0x67e5_5044_10b1_426f_9247_bb68_0e5f_e0c8);
    assert_eq!(Uuid::from_u128(id.as_u128()), id);
    assert_eq!(Uuid::from_u128_le(id.to_u128_le()), id);

    // Both are const fn, so a UUID can be built in a constant.
    const NIL: Uuid = Uuid::from_u128(0);
    assert!(NIL.is_nil());
}
```

**When to use it:** for arithmetic or range queries on the integer value, for
storing a UUID in a `u128` column, and in `const` contexts. Note that
`as_u128().to_string()` gives you a decimal number, not a UUID — if you want the
compact text form that is `simple()`.

### Inspecting one

#### `get_version` and `get_version_num`

Read the version nibble. Neither one validates anything: a handful of bytes you
made up will report a version just as happily as a generated UUID will.

```
use uuid::{Uuid, Version};

fn main() {
    assert_eq!(Uuid::new_v4().get_version(), Some(Version::Random));
    assert_eq!(Uuid::now_v7().get_version(), Some(Version::SortRand));
    assert_eq!(Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"x").get_version(), Some(Version::Sha1));
    assert_eq!(Uuid::new_v3(&Uuid::NAMESPACE_DNS, b"x").get_version(), Some(Version::Md5));
    assert_eq!(Uuid::new_v8([0; 16]).get_version(), Some(Version::Custom));

    // The nil and max UUIDs are versions too, not absences of one, so a
    // `get_version().is_some()` check does not mean "this is a real UUID".
    assert_eq!(Uuid::nil().get_version(), Some(Version::Nil));
    assert_eq!(Uuid::nil().get_version_num(), 0);
    assert_eq!(Uuid::max().get_version(), Some(Version::Max));
    assert_eq!(Uuid::max().get_version_num(), 15);
    assert_eq!(Uuid::from_u128(0).get_version(), Some(Version::Nil));
}
```

**When to use it:** to branch on what a UUID someone handed you actually is —
whether a timestamp can be recovered from it, whether it is safe to sort by. Do
not use it as a validity check: parsing already told you the value is
well-formed, and every 128-bit value has some version nibble.

#### `get_variant`

Reads the variant bits, which say which layout standard the value follows.

```
use uuid::{Uuid, Variant};

fn main() {
    assert_eq!(Uuid::new_v4().get_variant(), Variant::RFC4122);
    assert_eq!(Uuid::now_v7().get_variant(), Variant::RFC4122);

    // nil and max sit outside the RFC layout.
    assert_eq!(Uuid::nil().get_variant(), Variant::NCS);
    assert_eq!(Uuid::max().get_variant(), Variant::Future);
}
```

**When to use it:** rarely — if you are implementing a protocol that cares, or
sanity-checking a batch of IDs from an unknown source. Pairing it with
`get_version` is a reasonable "does this look generated by something modern"
heuristic, as long as you remember nil fails it.

#### `get_timestamp` and `get_node_id`

Pull back what a time-based UUID encoded. Both return `None` for the versions
that carry no such thing, so a [`Option`](../concepts/error-handling/option.md)
match here is a version check with the value attached.

```
use uuid::Uuid;

fn main() {
    let node = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    // v1 and v6 carry a node id; v7 does not.
    assert_eq!(Uuid::now_v1(&node).get_node_id(), Some(node));
    assert_eq!(Uuid::now_v6(&node).get_node_id(), Some(node));
    assert_eq!(Uuid::now_v7().get_node_id(), None);
    assert_eq!(Uuid::new_v4().get_node_id(), None);

    // v1 and v6 keep 100ns ticks; v7 keeps whole milliseconds. Converting a
    // v7 back is lossy by up to a millisecond, by construction.
    let (_, v7_nanos) = Uuid::now_v7().get_timestamp().unwrap().to_unix();
    assert_eq!(v7_nanos % 1_000_000, 0);

    assert!(Uuid::new_v4().get_timestamp().is_none());
    assert!(Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"x").get_timestamp().is_none());
}
```

**When to use it:** to get a creation time out of IDs you already have, which is
occasionally a gift when the table has no `created_at`. Do not treat it as
trustworthy: the timestamp is whatever the generating machine's clock said, and
with `new_v7` it is whatever the caller passed.

#### `Uuid::nil`, `Uuid::max`, `is_nil` and `Default`

The all-zero and all-ones values. `Default` is nil, which is the part that bites.

```
use uuid::Uuid;

fn main() {
    assert_eq!(Uuid::nil().to_string(), "00000000-0000-0000-0000-000000000000");
    assert_eq!(Uuid::max().to_string(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
    assert!(Uuid::nil().is_nil());
    assert!(Uuid::max().is_max());

    // Uuid::default() is the nil UUID, so a #[derive(Default)] struct gets a
    // silently invalid id rather than a missing one.
    assert_eq!(Uuid::default(), Uuid::nil());

    #[derive(Default)]
    struct Row {
        id: Uuid,
    }
    assert!(Row::default().id.is_nil()); // <- not a generated id
}
```

**When to use it:** `nil` as an explicit sentinel where a protocol defines one.
Otherwise be wary of it: `#[derive(Default)]` on a struct with a `Uuid` field
produces nil IDs that will pass every type check and parse cleanly, and will
collide with each other in your database. Prefer `Option<Uuid>` — or
`NonNilUuid`, below.

#### `NonNilUuid`

A [newtype](../concepts/types-data-modeling/the-newtype-pattern.md) that cannot
hold nil, which gives the compiler a niche to use.

```
use uuid::{NonNilUuid, Uuid};

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    let nn = NonNilUuid::try_from(id).unwrap();
    assert_eq!(nn.get(), id);
    assert_eq!(Uuid::from(nn), id);

    assert_eq!(
        NonNilUuid::try_from(Uuid::nil()).unwrap_err().to_string(),
        "the UUID is nil"
    );

    // The payoff: Option<Uuid> needs a discriminant byte, Option<NonNilUuid>
    // does not, because nil is free to mean None.
    assert_eq!(std::mem::size_of::<Option<Uuid>>(), 17);
    assert_eq!(std::mem::size_of::<Option<NonNilUuid>>(), 16);
}
```

**When to use it:** in a struct field or a function signature where nil is
meaningless, so that "this ID exists" is checked once at the boundary instead of
everywhere. The byte saved on `Option` is a real bonus in a large collection,
but the type-level guarantee is the reason to reach for it.

### Assembling one by hand

#### `Builder`

Takes raw bytes and applies version and variant markers, without pretending the
bytes came from the right source. Every generator above is a thin wrapper over
it.

```
use uuid::{Builder, Uuid, Variant, Version};

fn main() {
    // Markers applied to bytes you supply.
    let id = Builder::from_bytes([0u8; 16])
        .with_version(Version::Random)
        .with_variant(Variant::RFC4122)
        .into_uuid();
    assert_eq!(id.to_string(), "00000000-0000-4000-8000-000000000000");

    // The shorthands name the intent instead.
    assert_eq!(
        Builder::from_random_bytes([0xab; 16]).into_uuid().to_string(),
        "abababab-abab-4bab-abab-abababababab"
    );
    assert_eq!(
        Builder::from_md5_bytes([0; 16]).into_uuid().to_string(),
        "00000000-0000-3000-8000-000000000000"
    );
    assert_eq!(
        Builder::from_sha1_bytes([0; 16]).into_uuid().to_string(),
        "00000000-0000-5000-8000-000000000000"
    );
    assert_eq!(
        Builder::from_custom_bytes([0; 16]).into_uuid().to_string(),
        "00000000-0000-8000-8000-000000000000"
    );

    // from_bytes applies nothing at all, which is how you build nil.
    assert_eq!(Builder::from_bytes([0; 16]).into_uuid(), Uuid::nil());
}
```

**When to use it:** when you have your own randomness — a CSPRNG you already
trust, bytes from an HSM, a seeded generator in a test — and want a correctly
marked v4 without `getrandom`. `from_random_bytes` is also the way to get v4s
in `no_std` without wiring up a `getrandom` backend. For an ordinary v4, call
`new_v4` instead.

### Serialising

#### `serde` support and `uuid::serde::*`

With the `serde` feature, `Uuid` is `Serialize` and `Deserialize`. The
representation depends on the format: a string where the format is
human-readable, raw bytes where it is not. The modules under `uuid::serde`
override that per field.

```
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Plain {
    id: Uuid,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct AsSimple {
    #[serde(with = "uuid::serde::simple")]
    id: Uuid,
}

fn main() {
    let id = Uuid::try_parse("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();

    assert_eq!(
        serde_json::to_string(&Plain { id }).unwrap(),
        r#"{"id":"67e55044-10b1-426f-9247-bb680e5fe0c8"}"#
    );
    assert_eq!(
        serde_json::to_string(&AsSimple { id }).unwrap(),
        r#"{"id":"67e5504410b1426f9247bb680e5fe0c8"}"#
    );

    // The default reader is as permissive as try_parse.
    for text in [
        r#"{"id":"67e55044-10b1-426f-9247-bb680e5fe0c8"}"#,
        r#"{"id":"67e5504410b1426f9247bb680e5fe0c8"}"#,
        r#"{"id":"{67e55044-10b1-426f-9247-bb680e5fe0c8}"}"#,
        r#"{"id":"urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8"}"#,
    ] {
        assert_eq!(serde_json::from_str::<Plain>(text).unwrap().id, id);
    }

    // An annotated field is NOT: it reads only its own form. Switching a
    // field to `simple` for prettier output makes it reject every hyphenated
    // value already stored.
    assert!(serde_json::from_str::<AsSimple>(
        r#"{"id":"67e5504410b1426f9247bb680e5fe0c8"}"#
    ).is_ok());
    assert!(serde_json::from_str::<AsSimple>(
        r#"{"id":"67e55044-10b1-426f-9247-bb680e5fe0c8"}"#
    ).is_err()); // <- a one-way schema change

    // In a binary format the default is a length-prefixed byte sequence;
    // `compact` is a fixed sixteen bytes. The two are not wire-compatible.
    let default_bytes = postcard::to_allocvec(&Plain { id }).unwrap();
    assert_eq!(default_bytes.len(), 17);
    assert_eq!(default_bytes[0], 16); // <- the length prefix
}
```

**When to use it:** leave it on the default for JSON and anything else a person
reads, and for binary formats where you are not counting bytes. Switch to
`uuid::serde::compact` at the start of a project if the extra byte matters, not
later — and the same goes for `simple`, `braced` and `urn`, each of which makes
the field refuse the other forms. `serde_json` is used above; see
[`serde`](serde.md) and [`serde_json`](serde_json.md) for the derive machinery.

### Running without std

#### Feature wiring for `no_std` and WebAssembly

`default-features = false` drops `std`. What survives is everything that needs
no clock and no randomness.

```
use uuid::Uuid;

fn main() {
    // Buildable for thumbv7m-none-eabi with default-features = false:
    //
    //   uuid = { version = "1.29.2", default-features = false }
    //
    // parsing, formatting into a stack buffer, and the const macro.
    const ID: Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
    let mut buf = Uuid::encode_buffer();
    assert_eq!(
        ID.hyphenated().encode_lower(&mut buf),
        "67e55044-10b1-426f-9247-bb680e5fe0c8"
    );

    // v3, v5 and v8 need no entropy, so they work on bare metal too.
    assert_eq!(
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"example.org").to_string(),
        "aad03681-8b63-5304-89e0-8ca8f49461b5"
    );

    // v4 and v7 do NOT build for thumbv7m-none-eabi: getrandom stops the
    // build with "target is not supported. You may need to define a custom
    // backend". Supply your own bytes instead:
    //
    //   Builder::from_random_bytes(my_rng_bytes).into_uuid()
    //
    // On wasm32-unknown-unknown the crate itself refuses, with "to use
    // `uuid` on `wasm32-unknown-unknown`, specify a source of randomness
    // using one of the `js`, `rng-getrandom`, or `rng-rand` features".
    // Adding `js` is enough; it brings wasm-bindgen and js-sys.
    assert!(!ID.is_nil());
}
```

**When to use it:** `default-features = false` whenever you are embedding, and
pair it with `v5` or `Builder::from_random_bytes` rather than `v4`. For a
browser build, enabling `js` is the fix, and the compiler will tell you so
before you ship. `now_v1`, `now_v6` and `now_v7` all need `std` regardless,
because they read the system clock.

---
title: "use"
kind: keyword
embedded_support: full
groups: ["Modules & Visibility", "Modules, Crates & Visibility"]
related_concepts: [Modules, "Visibility & privacy"]
related_syntax: [mod, pub, self, as, "::"]
see_also: [mod, pub, self, as]
---

## Explanation

`use` brings a path into scope so it can be written shorter afterward. It
doesn't move, copy, or create anything — it's purely a shorthand for the
compiler's name resolution, resolved entirely at compile time.

The basic form names one path: `use std::collections::HashMap;` — after
this line, `HashMap` refers to `std::collections::HashMap` anywhere in the
scope the `use` is written in.

Several imports sharing a prefix can be grouped with braces:
`use std::io::{self, Read, Write};`. Inside the braces, `self` is a path
segment meaning "the group's own prefix path" — here, `io` itself, so this
line imports `io` (as `io`), `Read`, and `Write` all in one statement. That's
an unrelated meaning to `self` as a method receiver — see
[`self`](self.md) for that.

`as` renames an imported item: `use foo::Bar as Baz;` binds the item at
`foo::Bar` to the local name `Baz`. This lets two identically-named items
from different modules coexist in one scope, and is a completely
different use of `as` than its primary role of numeric/type casting
(`x as i64`) — see [`as`](as.md) for that.

A glob import — `use foo::*;` — brings every public item at `foo` into
scope at once. It's convenient but leaves the source of a name less
obvious at the call site, so it's mostly reserved for preludes and test
modules rather than everyday imports.

Marking a `use` with `pub` — `pub use` — doesn't just shorten the path
locally; because the import is itself `pub`, the imported path becomes
part of *this* module's own public surface, reachable by others through
the new location as well as the original one. See
[Modules](../../concepts/modules-crates-visibility/modules.md) for why
crates lean on `pub use` to decouple internal module layout from a
curated public API, and
[Visibility & privacy](../../concepts/modules-crates-visibility/visibility-and-privacy.md)
for what `pub` controls generally.

## Usage examples

### Importing `HashMap` with `use`

```
use std::collections::HashMap; // <- brings the full path into scope as `HashMap`

let mut ages: HashMap<&str, u32> = HashMap::new();
ages.insert("Priya", 34);
```

### Designing a public API

A crate re-exports two internal `Error` types under distinct public
names, renaming one of them with `as` so the two don't collide once
they're re-exported from the same place.

```
// src/lib.rs
mod transport;
mod storage;

pub use transport::Error as TransportError; // <- `as` renames the import, not a cast
pub use storage::Error as StorageError;     // <- avoids a name clash between two unrelated `Error` types

// src/transport.rs
#[derive(Debug)]
pub struct Error;

// src/storage.rs
#[derive(Debug)]
pub struct Error;
```

Renaming at the `use` site is the standard way two
identically-named items from different modules coexist in one scope, and
re-exporting each under a distinct, descriptive public name keeps the
crate's internal module names from leaking out as one ambiguous `Error`,
in the spirit of the
[API Guidelines' naming conventions](https://rust-lang.github.io/api-guidelines/naming.html).

### Working with collections

Code that needs both a set and a map from `std::collections` groups them
in one `use` line instead of writing two separate imports.

```
use std::collections::{HashMap, HashSet}; // <- one `use` line, grouped import

let mut unique_tags: HashSet<String> = HashSet::new();
let mut tag_counts: HashMap<String, u32> = HashMap::new();

unique_tags.insert("rust".to_string());
*tag_counts.entry("rust".to_string()).or_insert(0) += 1;
```

Grouping related imports from a shared prefix keeps the
import block scanning as one unit rather than one line per type — the
form `rustfmt` itself normalizes multi-item imports from the same module
into.

## Embedded Rust Notes

**Full support.** `use` is pure compile-time name resolution with no
runtime representation, so it works identically in a `#![no_std]` crate —
importing from `core`/`alloc` instead of `std` is the only difference,
and it's a difference in *which* paths exist, not in how `use` itself
behaves.

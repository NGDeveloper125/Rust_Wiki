---
title: "rustversion"
version: "1.0.23"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "yes"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-31"
summary: "Attribute macros for conditional compilation on the compiler version — `#[rustversion::since(1.80)]` on an item, which is the `#[cfg]` predicate Rust does not give you. Zero dependencies, and an MSRV of 1.31 so it runs on the old compilers it exists to support."
domain: "Build scripts & tooling"
categories: ["development-tools::build-utils", "no-std", "no-std::no-alloc"]
repository: "https://github.com/dtolnay/rustversion"
docs: "https://docs.rs/rustversion/1.0.23/rustversion/"
---

## Overview

Rust has no `#[cfg]` predicate for the compiler version. There is no
`#[cfg(version = "1.80")]` on stable, and a `cfg` name you invent is not a
version test — so a library that wants to offer an impl only when the compiler
is new enough has nothing built in to ask with. Writing the obvious thing fails
outright:

```
// no-compile — demonstrates the error rustversion exists to work around
#[cfg(rustversion::since(1.70))] // <- error[E0537]: invalid predicate
fn gated() {}
```

`rustversion` supplies the missing test as a set of attribute macros. Put one on
an item and the item survives or vanishes depending on the compiler doing the
build.

```
#[rustversion::since(1.80)]
fn describe() -> &'static str {
    "compiled by 1.80 or newer"
}

#[rustversion::before(1.80)]
fn describe() -> &'static str {
    "compiled by something older"
}

// Exactly one of the two above exists, so this always resolves.
assert!(describe().starts_with("compiled by"));
```

There is also an expression form, which is the one to reach for inside a
function or a build script:

```
let modern: bool = rustversion::cfg!(since(1.70));
assert!(modern);

assert_eq!(
    rustversion::cfg!(stable) && rustversion::cfg!(nightly),
    false // <- a compiler is on exactly one channel
);
```

**The selectors are mutually exclusive only if you make them so.** This is the
one structural thing to understand, because the failure is a confusing error
rather than wrong behaviour. The attributes do not form an `if`/`else` chain —
each is evaluated on its own, so two overlapping conditions on the same name
leave two copies of the item:

```
// no-compile — error[E0428]: the name `dup` is defined multiple times
#[rustversion::since(1.0)]
fn dup() {}

#[rustversion::before(1.99)] // <- also true, so `dup` is defined twice
fn dup() {}
```

The idiom that avoids it is `since`/`before` on the same version, as in the
first example, or `not` wrapping the exact condition used by the other arm.

**It applies to items, and only items.** Attribute macros on statements are
still unstable in Rust itself, so this is rejected with `error[E0658]: custom
attributes cannot be applied to statements`:

```
// no-compile — error[E0658]: custom attributes cannot be applied to statements
fn f() {
    #[rustversion::since(1.70)] // <- not allowed on a `let`
    let x = 1;
    let _ = x;
}
```

Gate a function, impl, `use`, struct or module; use `cfg!` for anything smaller.

**Date selectors mean nightly dates, not calendar dates.** Easy to misread, and
quietly wrong if you do. `since(2020-01-01)` asks "is this a nightly from that
date or later", so it is **false on every stable compiler** no matter how
recently that stable was released:

```
// On stable 1.96.0, released in 2026:
assert_eq!(rustversion::cfg!(since(2020-01-01)), false);
assert_eq!(rustversion::cfg!(before(2020-01-01)), true);
assert_eq!(rustversion::cfg!(stable), true);
```

Release numbers also have a fixed shape — `1.80` or `1.80.1`, with the major
version `1`. Anything else is a compile error from the macro, which at least
fails loudly.

**How it knows.** A build script runs `rustc --version` and bakes the answer in,
honouring `RUSTC` and `RUSTC_WRAPPER`, with special handling for
`clippy-driver`. The version is therefore the compiler *building your crate*,
which is what you want; cross-compiling does not change it, since only the
target differs.

**When not to use it.** Most code should not need this. Raising `rust-version`
in `Cargo.toml` is simpler than supporting both sides of a version split, and
costs nothing but an honest MSRV. Reach for `rustversion` when you genuinely
must span compilers — a widely-depended-on library, or code offering a
`nightly`-only fast path. Two neighbours worth knowing: `autocfg` probes whether
a snippet *compiles*, so it detects capability rather than a version number and
survives backports; `version_check` does version tests but only from a build
script, with no attribute macros.

The dependency cost is as low as it goes: **zero dependencies**, a proc macro
written without [`syn`](syn.md) or [`quote`](quote.md). It emits no runtime code,
so `no_std` is a non-question. Its own **MSRV is 1.31** on edition 2018 —
deliberately ancient, because a crate for supporting old compilers cannot
require a new one.

## When to use it

### Use case: Offering an API that only exists on newer compilers

The reason the crate exists: a trait impl or type you cannot name on an older
toolchain.

```
pub struct Wrapper<T>(pub T);

// Only compiled where the compiler has the trait being implemented.
#[rustversion::since(1.33)]
impl<T> Wrapper<T> {
    pub fn pinned(self: std::pin::Pin<&Self>) -> &T {
        &self.get_ref().0
    }
}

// The rest of the type is unconditional.
impl<T> Wrapper<T> {
    pub fn get(&self) -> &T {
        &self.0
    }
}

let w = Wrapper(7);
assert_eq!(*w.get(), 7);
```

**Why it fits:** the `impl` block simply does not exist on a compiler that would
reject it, so old toolchains keep building and new ones get the extra method —
without a Cargo feature that users have to know to turn on, and without
splitting the crate. Note that only the conditional part is gated: gate the
smallest item that will not compile, because everything inside a gated item
disappears with it.

### Use case: Making a function `const` as soon as the compiler allows

Conditional *attributes*, which is a different job from conditional items.

```
// `const` is applied from 1.32 onward and omitted before it.
#[rustversion::attr(since(1.32), const)]
fn double(n: u32) -> u32 {
    n * 2
}

// So on a new enough compiler it works in a const context.
const EIGHT: u32 = double(4);
assert_eq!(EIGHT, 8);
assert_eq!(double(5), 10);
```

**Why it fits:** writing this with gated items means two copies of the body, one
`const` and one not, which then drift. `attr` keeps a single definition and
varies only the attribute on it. `const` is a special case the macro recognises
even though it is a keyword rather than an attribute; anything that is an actual
attribute — `derive(...)`, `repr(packed(2))`, `inline` — works the same way.

### Use case: Turning a version test into a `cfg` for the whole crate

The build-script route, for when one decision should be visible everywhere.

```
// build.rs
fn main() {
    // Declare the cfg first, or every use of it warns under unexpected_cfgs.
    println!("cargo::rustc-check-cfg=cfg(has_modern)");

    if rustversion::cfg!(since(1.70)) {
        println!("cargo::rustc-cfg=has_modern");
    }
}
```

Then `#[cfg(has_modern)]` works throughout the crate like any other `cfg`,
including in positions no attribute macro can reach — inside `#[cfg_attr]`, on
match arms, in `cfg!()`.

**Why it fits:** one evaluation produces a name the rest of the crate tests
normally, which beats repeating `#[rustversion::since(1.70)]` on thirty items.
Two requirements are easy to miss. `rustversion` must be under
**`[build-dependencies]`**, not `[dependencies]`, because the test runs in the
build script. And since Rust 1.80, a `cfg` name cargo sets must also be declared
with `cargo::rustc-check-cfg` or `rustc` warns `unexpected cfg condition name`
at every use — the upstream example predates that lint, so copying it verbatim
gives you a warning per use site.

## API map

Ten macros: selectors that test the compiler, combinators over them, one for
applying an attribute, and one expression form.

### Version and channel selectors

#### `since` and `before`

A lower and upper bound on the release.

```
// since is inclusive of the named release.
#[rustversion::since(1.96)]
fn from_196() -> bool {
    true
}

// before is its exact negation, so the pair is always exhaustive.
#[rustversion::before(1.96)]
fn from_196() -> bool {
    false
}

assert!(from_196());

// A patch release is accepted too.
assert_eq!(rustversion::cfg!(since(1.80.1)), true);
```

**When to use it:** `since` for almost everything — it is the question "is the
compiler new enough", which is the question you actually have. Pair it with
`before` on the *same* version to get two arms that cannot overlap and cannot
both be missing. `since` also applies to beta and nightly, so
`since(1.96)` is true on a nightly that will become 1.97; that is usually what
you want, and `stable(1.96)` is the macro to use when it is not.

#### `stable`

Any stable compiler, or one exact stable release.

```
#[rustversion::stable]
fn on_stable() -> bool {
    true
}

#[rustversion::not(stable)]
fn on_stable() -> bool {
    false
}

assert!(on_stable());

// With an argument it is that one release and nothing else.
assert_eq!(rustversion::cfg!(stable(1.96)), true);
assert_eq!(rustversion::cfg!(stable(1.95)), false);
```

**When to use it:** bare `stable` to exclude pre-release toolchains — for
instance to suppress a lint that only fires on nightly. The argument form is
narrow by design: `stable(1.96)` is an equality test, not a bound, and it is
false on the 1.96 beta and on nightly. Reach for `since` when you mean "this
version or later", which is nearly always.

#### `beta`

Any beta compiler.

```
assert_eq!(rustversion::cfg!(beta), false);

// Useful mostly as part of a wider condition.
assert!(rustversion::cfg!(any(stable, beta)));
```

**When to use it:** seldom on its own, since code rarely cares about beta
specifically. Its real use is in `any(stable, beta)` to mean "a released
compiler, not nightly", and as the channel you check when you are testing
whether something about to be stabilised has landed.

#### `nightly`

Any nightly or dev build, or one exact nightly date.

```
#[rustversion::nightly]
fn fast_path() -> &'static str {
    "nightly-only implementation"
}

#[rustversion::not(nightly)]
fn fast_path() -> &'static str {
    "portable implementation"
}

assert_eq!(fast_path(), "portable implementation");
assert_eq!(rustversion::cfg!(nightly), false);
```

**When to use it:** to gate code behind an unstable feature, which is its
commonest job — a `#![feature(...)]` path with a stable fallback. The date form
`nightly(2026-01-01)` tests one specific nightly and is for working around a
known-broken build. Remember that `since` with a date is also nightly-only, so
neither helps you ask about a stable compiler's release date.

### Combining selectors

#### `not`

Negation.

```
#[rustversion::not(nightly)]
fn no_unstable_features() -> bool {
    true
}

assert!(no_unstable_features());
assert_eq!(rustversion::cfg!(not(stable)), false);
```

**When to use it:** to write the complement of whatever condition the other arm
uses, which is how you keep a pair of definitions from overlapping. When the
condition is a single `since`, prefer `before` over `not(since(..))` — it reads
better and means exactly the same thing. `not` earns its place over compound
conditions, where there is no shorter spelling.

#### `any` and `all`

Disjunction and conjunction.

```
// Any: a released compiler.
#[rustversion::any(stable, beta)]
fn released() -> bool {
    true
}

#[rustversion::not(any(stable, beta))]
fn released() -> bool {
    false
}

assert!(released());

// All: a closed range.
assert_eq!(rustversion::cfg!(all(since(1.70), before(1.99))), true);
assert_eq!(rustversion::cfg!(any(nightly, beta)), false);
```

**When to use it:** `all(since(a), before(b))` is the useful shape — a window in
which some bug or behaviour existed, which is exactly what you need when working
around a regression that was later fixed. `any` is mostly the channel test
above. They nest, so `not(all(..))` and `any(not(..), ..)` are both available;
pick whichever states the intent more plainly, since the compiler cannot help you
read it back.

### Conditional attributes

#### `attr`

Applies an attribute when the selector matches.

```
// A keyword, recognised specially.
#[rustversion::attr(since(1.32), const)]
fn tripled(n: u32) -> u32 {
    n * 3
}

// An ordinary attribute.
#[rustversion::attr(since(1.0), derive(Debug))]
struct Point {
    x: i32,
}

const NINE: u32 = tripled(3);
assert_eq!(NINE, 9);
assert_eq!(format!("{:?}", Point { x: 1 }), "Point { x: 1 }");
```

**When to use it:** whenever the *item* is the same across versions and only its
decoration differs — becoming `const`, changing `repr`, adding a `derive` that
did not exist yet, or applying an `allow` for a lint introduced in a later
release. It is `cfg_attr` with a version predicate, and it is the right answer
whenever the alternative would be two near-identical copies of a body.

### Expression form

#### `cfg!`

The selectors as a `bool` expression.

```
// Expands to a literal true or false, so it works anywhere a bool does.
if rustversion::cfg!(since(1.70)) {
    assert!(true);
}

const MODERN: bool = rustversion::cfg!(since(1.70));
assert!(MODERN);

// Including where a constant is required.
let buffer = [0u8; if rustversion::cfg!(since(1.70)) { 2 } else { 1 }];
assert_eq!(buffer.len(), 2);
```

**When to use it:** inside function bodies and build scripts — anywhere the
attribute macros cannot go, which per the overview includes every statement and
expression. It expands to a literal, so it is usable in `const` initialisers and
array lengths, and the branch not taken is still type-checked, unlike a gated
item that disappears. That last point cuts both ways: it keeps dead branches
honest, and it means `cfg!` cannot hide code the older compiler would reject.
For that you need a gated item.

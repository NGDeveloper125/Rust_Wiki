---
title: "semver"
version: "1.0.28"
publisher: "David Tolnay (dtolnay)"
publisher_url: "https://crates.io/users/dtolnay"
no_std: "optional"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-10-15"
summary: "Parses and compares version numbers, and evaluates the requirement syntax Cargo uses — `^1.2`, `~1.2.3`, `>=1.0, <2.0`. The same implementation Cargo itself resolves dependencies with."
domain: "Build scripts & tooling"
categories: ["data-structures", "no-std"]
repository: "https://github.com/dtolnay/semver"
docs: "https://docs.rs/semver/1.0.28/semver/"
---

## Overview

Two types carry the crate. `Version` is a parsed version number; `VersionReq` is
a parsed requirement that versions either satisfy or do not.

```
use semver::{Version, VersionReq};

let installed = Version::parse("1.4.2").unwrap();
let required = VersionReq::parse("^1.2").unwrap();

assert!(required.matches(&installed));

// Comparison follows SemVer precedence, so this ordering is meaningful.
assert!(Version::parse("1.0.0-alpha").unwrap() < Version::parse("1.0.0").unwrap());
assert!(Version::parse("1.9.0").unwrap() < Version::parse("1.10.0").unwrap());
```

The description on crates.io is precise and worth taking literally: this is
*Cargo's flavour* of semantic versioning. `VersionReq` implements the syntax from
`Cargo.toml`, which is not what semver.org specifies — semver.org defines version
numbers and says nothing about `^` or `~`. So the matching rules here are
Cargo's rules, and if you are comparing against another ecosystem's resolver
(npm's, say) the differences are real.

**One behaviour to know before you use it, because it is a trap.** SemVer says
build metadata must be ignored when comparing precedence. This crate's
`PartialEq` and `Ord` do *not* ignore it:

```
use semver::Version;

let a = Version::parse("1.0.0+build1").unwrap();
let b = Version::parse("1.0.0+build2").unwrap();

// The derived comparisons distinguish them...
assert!(a != b);
assert!(a < b);

// ...while SemVer precedence, which is what the spec defines, does not.
assert_eq!(a.cmp_precedence(&b), std::cmp::Ordering::Equal);
```

Both are defensible — `Eq` and `Ord` are total and consistent, which they would
not be if build metadata were dropped — but the default operators are not the
spec's comparison. If you are deduplicating versions or checking "is this the
same release", use `cmp_precedence`.

**The second thing to know is that requirements exclude pre-releases.** `^1.0.0`
does not match `1.0.1-alpha`, and neither does `*`. A pre-release is only
matched by a comparator that itself names a pre-release at the same
major.minor.patch. This is deliberate and matches Cargo: it is why adding a
dependency never silently pulls an alpha.

It is a dtolnay crate, with the usual consequences: no required dependencies, a
small API that has not broken since 1.0, MSRV 1.68, `no_std` with `alloc` by
turning off the default `std` feature, and optional `serde` support. If you only
need to compare two version strings it is more than you need — but version
comparison done by hand on string splits is wrong in ways that take a while to
notice, starting with `1.10.0` sorting before `1.9.0`.

## When to use it

### Use case: Checking an installed or reported version against a requirement

The common case: something tells you its version, and you decide whether it will
do.

```
use semver::{Version, VersionReq};

fn is_supported(reported: &str) -> Result<bool, semver::Error> {
    // Caret: any 1.x from 1.70 up, but not 2.0.
    let supported = VersionReq::parse("^1.70")?;
    let version = Version::parse(reported)?;
    Ok(supported.matches(&version))
}

assert!(is_supported("1.70.0").unwrap());
assert!(is_supported("1.94.1").unwrap());
assert!(!is_supported("1.69.9").unwrap());
assert!(!is_supported("2.0.0").unwrap());
assert!(is_supported("not a version").is_err());
```

**Why it fits:** the requirement is expressed in the syntax everyone already
reads from `Cargo.toml`, so `"^1.70"` needs no explanation in a config file or
an error message. Doing it by hand means reimplementing the caret rule, which
has a special case for `0.x` that is easy to miss. Note the `Result`: a version
string from outside your program can be anything, and the parse error says
exactly what was wrong.

### Use case: Sorting releases and finding the newest

Ordering that gets pre-releases right.

```
use semver::Version;

let mut releases: Vec<Version> = ["1.10.0", "1.9.0", "2.0.0-rc.1", "2.0.0", "1.9.1"]
    .iter()
    .map(|text| Version::parse(text).unwrap())
    .collect();

releases.sort();

let ordered: Vec<String> = releases.iter().map(ToString::to_string).collect();
assert_eq!(
    ordered,
    vec!["1.9.0", "1.9.1", "1.10.0", "2.0.0-rc.1", "2.0.0"]
);

// The newest stable release, skipping pre-releases.
let latest_stable = releases.iter().filter(|v| v.pre.is_empty()).max().unwrap();
assert_eq!(latest_stable.to_string(), "2.0.0");
```

**Why it fits:** two things here are wrong under string sorting and right here —
`1.10.0` after `1.9.1` because the components are numbers, and `2.0.0-rc.1`
before `2.0.0` because a pre-release precedes its release. Both are the kind of
bug that survives review and then surfaces as a release notification pointing
at the wrong version. `pre.is_empty()` is the idiomatic stable-only filter.

### Use case: Inspecting a requirement rather than just applying it

Reading the parsed structure, for tooling that reports on dependencies.

```
use semver::{Op, VersionReq};

let requirement = VersionReq::parse(">=1.2, <1.5").unwrap();

// A requirement is a list of comparators, all of which must hold.
assert_eq!(requirement.comparators.len(), 2);

let first = &requirement.comparators[0];
assert_eq!(first.op, Op::GreaterEq);
assert_eq!(first.major, 1);
assert_eq!(first.minor, Some(2));
assert_eq!(first.patch, None); // ">=1.2" names no patch

// Caret is what a bare version means.
let bare = VersionReq::parse("1.2.3").unwrap();
assert_eq!(bare.comparators[0].op, Op::Caret);
```

**Why it fits:** anything that audits or rewrites dependency specifications — a
lint that forbids wildcards, a tool that tightens bounds before a release, a
report on how pinned a tree is — needs the parsed form rather than the string.
The `Option` on `minor` and `patch` is the useful part: it preserves whether the
author wrote `^1`, `^1.2` or `^1.2.3`, which changes what the requirement means
and which a normalised form would lose.

## API map

Four types and an enum. The parsing entry points all look the same, so the
entries below concentrate on what each type is for and where the semantics are
not obvious.

### Versions

#### `Version::parse` and `Version::new`

Getting a version from a string, or building one.

```
use semver::Version;

// Parsing requires all three components: "1.2" is an error.
let parsed = Version::parse("1.2.3-alpha.1+build.5").unwrap();
assert_eq!((parsed.major, parsed.minor, parsed.patch), (1, 2, 3));
assert_eq!(parsed.pre.as_str(), "alpha.1");
assert_eq!(parsed.build.as_str(), "build.5");

assert!(Version::parse("1.2").is_err());

// new is const and takes the three numbers, leaving pre and build empty.
let built = Version::new(1, 2, 3);
assert_eq!(built.to_string(), "1.2.3");
assert!(built.pre.is_empty());
```

**When to use it:** `parse` for anything from outside — a CLI argument, an HTTP
response, a file. `new` when you are constructing one, and it is `const`, so a
baseline version can be a `const` item. The strictness of `parse` is
deliberate: `1.2` is not a version under SemVer, it is a requirement, and
accepting it would hide the distinction the two types exist to keep.

#### Comparison, and `cmp_precedence`

The two different notions of equality.

```
use semver::Version;
use std::cmp::Ordering;

let alpha = Version::parse("1.0.0-alpha").unwrap();
let beta = Version::parse("1.0.0-beta").unwrap();
let release = Version::parse("1.0.0").unwrap();

// Pre-releases sort before the release, and among themselves by identifier.
assert!(alpha < beta);
assert!(beta < release);

// Build metadata: compared by the operators, ignored by precedence.
let one = Version::parse("1.0.0+a").unwrap();
let two = Version::parse("1.0.0+b").unwrap();
assert_eq!(one.cmp(&two), Ordering::Less);
assert_eq!(one.cmp_precedence(&two), Ordering::Equal);
```

**When to use it:** `Ord` for sorting and for `BTreeMap` keys, where a total
order is what you need and the build-metadata tiebreak is harmless. Switch to
`cmp_precedence` whenever the question is "are these the same release" or "which
of these is newer" in the spec's sense — comparing a lockfile entry against a
registry version, or deduplicating. Using `==` there will treat two builds of
one release as different versions.

#### `Prerelease` and `BuildMetadata`

The two optional trailing parts.

```
use semver::{BuildMetadata, Prerelease, Version};

let mut version = Version::new(2, 0, 0);
assert_eq!(version.to_string(), "2.0.0");

// Both are validated on construction, so an invalid identifier is an error.
version.pre = Prerelease::new("rc.1").unwrap();
version.build = BuildMetadata::new("20261015").unwrap();
assert_eq!(version.to_string(), "2.0.0-rc.1+20261015");

assert!(Prerelease::new("has space").is_err());

// EMPTY is the absent case, and is const.
assert!(Prerelease::EMPTY.is_empty());
assert!(BuildMetadata::EMPTY.is_empty());
```

**When to use it:** `pre.is_empty()` is the check for "is this a stable
release", and it is the one you will reach for most. Constructing them matters
when you are generating versions — a nightly build stamping a date into build
metadata, a release tool bumping to `-rc.1`. They are separate types rather than
`Option<String>` because the identifier grammar is validated, and because
`EMPTY` lets the common case stay allocation-free.

### Requirements

#### `VersionReq::parse` and `matches`

Evaluating Cargo's requirement syntax.

```
use semver::{Version, VersionReq};

fn matches(requirement: &str, version: &str) -> bool {
    VersionReq::parse(requirement)
        .unwrap()
        .matches(&Version::parse(version).unwrap())
}

// Caret, and what a bare version means: compatible up to the next breaking bump.
assert!(matches("^1.2.3", "1.9.0"));
assert!(!matches("^1.2.3", "2.0.0"));
assert!(matches("1.2.3", "1.9.0")); // identical to ^1.2.3

// Below 1.0 the "breaking" position shifts left, which is the special case.
assert!(matches("^0.2.3", "0.2.9"));
assert!(!matches("^0.2.3", "0.3.0"));
assert!(!matches("^0.0.3", "0.0.4")); // ^0.0.x is effectively exact

// Tilde pins one level deeper than caret.
assert!(matches("~1.2.3", "1.2.9"));
assert!(!matches("~1.2.3", "1.3.0"));

// Several comparators are an AND.
assert!(matches(">=1.2, <1.5", "1.4.0"));
assert!(!matches(">=1.2, <1.5", "1.5.0"));
```

**When to use it:** wherever a human writes a version range. The `0.x` rule is
the one worth committing to memory, because it is where intuition fails:
`^0.2.3` allows patch and nothing more, and `^0.0.3` allows nothing at all,
since every `0.0.x` release is treated as potentially breaking. A comma is
conjunction, not disjunction — there is no OR in this syntax.

#### Pre-release matching

Why an alpha never shows up uninvited.

```
use semver::{Version, VersionReq};

fn matches(requirement: &str, version: &str) -> bool {
    VersionReq::parse(requirement)
        .unwrap()
        .matches(&Version::parse(version).unwrap())
}

// An ordinary requirement excludes pre-releases entirely...
assert!(!matches("^1.0.0", "1.0.1-alpha"));
assert!(!matches("*", "1.0.0-alpha"));

// ...unless the comparator itself names one at the same version.
assert!(matches(">=1.0.0-alpha", "1.0.0-beta"));
assert!(matches("^1.0.0-alpha", "1.0.0-beta"));
```

**When to use it:** as the explanation when a requirement "should" match and
does not. The rule is that opting into pre-releases is explicit, per version:
naming `1.0.0-alpha` opts into pre-releases of `1.0.0` only, and says nothing
about `1.1.0-alpha`. If you are writing a resolver or a mirror of one, this is
the behaviour to replicate rather than the one to fix.

#### `VersionReq::STAR` and the parsed form

The permissive requirement, and reading one apart.

```
use semver::{Op, Version, VersionReq};

// STAR is const and matches any release.
assert!(VersionReq::STAR.matches(&Version::new(0, 1, 0)));
assert!(VersionReq::STAR.matches(&Version::new(9, 9, 9)));
assert!(VersionReq::STAR.comparators.is_empty());

// A wildcard written out parses to a single Wildcard comparator.
let partial = VersionReq::parse("1.*").unwrap();
assert_eq!(partial.comparators[0].op, Op::Wildcard);
assert_eq!(partial.comparators[0].major, 1);
```

**When to use it:** `STAR` as a default when no requirement was supplied — it is
`const`, so it costs nothing and reads better than parsing `"*"` at run time.
The parsed form is for tooling: `Op` has eight variants (`Exact`, `Greater`,
`GreaterEq`, `Less`, `LessEq`, `Tilde`, `Caret`, `Wildcard`), and an empty
`comparators` list is how "no constraint" is represented, which a lint looking
for unbounded dependencies needs to recognise.

#### `Comparator`

A single clause of a requirement.

```
use semver::{Comparator, Op, Version};

// Parsing one clause on its own, rather than a whole requirement.
let comparator = Comparator::parse("^1.2").unwrap();

assert_eq!(comparator.op, Op::Caret);
assert_eq!(comparator.major, 1);
assert_eq!(comparator.minor, Some(2));
assert_eq!(comparator.patch, None);

// It matches on its own too.
assert!(comparator.matches(&Version::new(1, 5, 0)));
assert!(!comparator.matches(&Version::new(2, 0, 0)));
```

**When to use it:** when you are building or rewriting requirements
programmatically and want to work a clause at a time — tightening a lower
bound, dropping an upper one. The fields are public and writable, so a
`Comparator` can be constructed or edited directly rather than round-tripped
through a string, which is the point of exposing the structure.

### Errors

#### `semver::Error`

What a failed parse tells you.

```
use semver::Version;

let error = Version::parse("1.2").unwrap_err();
assert_eq!(
    error.to_string(),
    "unexpected end of input while parsing minor version number"
);

let other = Version::parse("not a version").unwrap_err();
assert!(other.to_string().contains("major version number"));
```

**When to use it:** pass the message straight through to the user. It names the
component being parsed and what went wrong, which is specific enough to act on
without you writing a wrapper message — and more specific than "invalid
version", which is what hand-rolled parsing usually manages. One `Error` type
covers `Version`, `VersionReq`, `Comparator`, `Prerelease` and `BuildMetadata`,
and it implements `std::error::Error`, so `?` into
[`anyhow`](anyhow.md) or a [`thiserror`](thiserror.md) enum works directly.

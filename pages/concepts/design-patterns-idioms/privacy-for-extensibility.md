---
title: "Privacy for extensibility"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Idioms", "Design Patterns & Idioms", "Encapsulation"]
related_syntax: [pub]
see_also: ["Visibility & privacy (pub and friends)", "Exhaustiveness checking", "Constructor functions (new() convention)", "The newtype pattern"]
---

## Explanation

[Visibility & privacy](../modules-crates-visibility/visibility-and-privacy.md)
covers privacy as encapsulation — hiding a field so outside code can't
put it in an invalid state. This idiom is the same mechanism aimed at a
different, narrower goal: keeping the *ability to add fields later*
without that addition becoming a breaking change for every downstream
crate.

The problem it solves: if a public struct's fields are *all* `pub`,
outside code is free to build it with a struct literal —
`Config { host: "x".into(), port: 80 }`. That looks harmless until the
struct needs a new field next release. Adding `timeout_ms: u64` breaks
every one of those literals at every call site across every downstream
crate, because none of them mention the new field — this is exactly the
kind of accidental breaking change [semver](../modules-crates-visibility/dependency-management-and-semver.md)
is supposed to protect callers from.

The fix costs nothing on the type's actual data: give the struct one
field that is *not* `pub` — sometimes a real private field, sometimes a
zero-sized marker field added purely for this purpose — alongside a
[public constructor](constructor-functions-new.md). With even a single
private field present, a struct literal naming every field is no longer
legal from outside the module (there's a field it isn't allowed to
name), so the only way to build the type is through the constructor.
Every future field addition then only has to update the constructor and
its callers who already go through it, not every struct-literal call
site across the ecosystem.

This is a close cousin of `#[non_exhaustive]` on
[enums](../pattern-matching/exhaustiveness-checking.md), which solves the
same "leave room to grow without a breaking change" problem for enum
variants and struct-literal construction directly via an attribute rather
than a manufactured private field. Where `#[non_exhaustive]` is the
built-in, self-documenting tool for structs and enums alike, "one private
field" is the idiom that predates it and still shows up in code that
wants finer control — for instance, a field that's genuinely private
implementation state (not just a marker) doing double duty as the
extensibility guard, with no attribute involved at all.

## Basic usage example

```
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    _reserved: (), // <- private: blocks struct-literal construction from outside the module
}

impl ConnectionConfig {
    pub fn new(host: String, port: u16) -> Self {
        ConnectionConfig { host, port, _reserved: () }
    }
}

let config = ConnectionConfig::new("db.internal".to_string(), 5432);
// ConnectionConfig { host: "x".into(), port: 1, _reserved: () } // would fail: _reserved isn't nameable outside the module
```

## Best practices & deeper information

### Scenario: Designing a public API

A library's `ClientOptions` struct exposes a couple of commonly-tuned
knobs today, but its author expects to add more later (a retry policy, a
proxy setting) — a private field keeps every current field `pub` for
convenient reading while still forcing construction through `new`.

```
pub struct ClientOptions {
    pub timeout_ms: u64,
    pub max_retries: u32,
    _extensible: (), // <- private: exists only to block exhaustive struct-literal construction
}

impl ClientOptions {
    pub fn new(timeout_ms: u64, max_retries: u32) -> Self {
        ClientOptions { timeout_ms, max_retries, _extensible: () }
    }
}

// downstream crate:
let opts = ClientOptions::new(5_000, 3);
println!("{} {}", opts.timeout_ms, opts.max_retries);
// ClientOptions { timeout_ms: 5_000, max_retries: 3 } // would fail to compile: `_extensible` can't be named here
```

**Why this way:** adding a field to `ClientOptions` next release — say
`pub proxy: Option<String>` — only requires updating `new` and its
callers, not every struct literal across the ecosystem, because no
outside code could ever construct one exhaustively in the first place;
the
[Rust Design Patterns](https://rust-unofficial.github.io/patterns/idioms/priv-extend.html)
book documents this as the idiomatic way to keep a struct extensible
before `#[non_exhaustive]` existed, and it's still used where a real
private field does the job naturally.

### Scenario: Creating a new object

Because outside code can no longer build `ClientOptions` directly, `new`
isn't just a convenience constructor here — it's the *only* way to
produce one, which is exactly the point.

```
pub struct RateLimiter {
    pub requests_per_sec: u32,
    _guard: (), // <- private: forces every caller through RateLimiter::new
}

impl RateLimiter {
    pub fn new(requests_per_sec: u32) -> Self { // <- the sole entry point for building this type
        RateLimiter { requests_per_sec, _guard: () }
    }
}

let limiter = RateLimiter::new(100);
println!("{}", limiter.requests_per_sec);
```

**Why this way:** pairing a privacy-for-extensibility field with a
constructor is what makes the idiom work end to end — the private field
closes off struct-literal construction, and `new` reopens exactly one
supported path, so the type can grow new fields behind that single
entry point without ever breaking a caller who used it.

## Embedded Rust Notes

**Full support.** The extra field is zero-sized and exists purely for the
compiler's benefit — it adds no runtime footprint and behaves identically
under `#![no_std]`. This idiom is common in embedded HAL crates for
peripheral configuration structs, which tend to gain new optional fields
(a new clock source, a new pin mode) across HAL versions.

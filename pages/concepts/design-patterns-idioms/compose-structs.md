---
title: "Compose structs"
area: "Design Patterns & Idioms"
embedded_support: full
groups: ["Design Patterns & Idioms", "Composition"]
related_syntax: []
see_also: ["Composition over inheritance", "The builder pattern", "Structs"]
---

## Explanation

"Compose structs" is the data-modeling half of
[composition over inheritance](composition-over-inheritance.md): break a
struct that has accumulated many only loosely related fields — a "god
struct" — into several smaller structs grouped by what actually changes
together, then compose the original type back out of them as fields. A
`ServerConfig` with ten flat fields (`host`, `port`, `log_level`,
`log_file`, `max_connections`, `db_url`, `db_pool_size`, …) reads as one
undifferentiated blob; the same data as `ServerConfig { network:
NetworkConfig, logging: LoggingConfig, database: DatabaseConfig }` reads
as three clearly named concerns, each small enough to understand,
construct, and pass around on its own.

The payoff isn't just readability. A smaller sub-struct can carry its
own `impl` block with methods that only make sense for that concern
(`NetworkConfig::bind_address(&self)`), can implement `Default`
independently so unrelated concerns don't need to agree on one shared
default, and — most importantly — can be constructed, validated, and
reused on its own: a `DatabaseConfig` extracted this way can be shared
between the server binary and a separate migration tool, without either
one depending on `ServerConfig` as a whole. A flat god struct forces
every consumer to either depend on the entire thing or duplicate the
fields it actually needs.

This is the same instinct behind normalizing a database schema — group
fields by what changes together, not just by what happens to exist on
the same conceptual "thing" — applied to a Rust type instead of a table.
It's a distinct concern from
[composition over inheritance](composition-over-inheritance.md): that
page is about sharing *behavior* across types via traits, while this
idiom is about grouping *fields* into smaller, independently meaningful
[structs](../types-data-modeling/structs.md). The two are usually
applied together, since a well-factored sub-struct is also a natural
place to attach its own methods.

## Basic usage example

```
struct NetworkConfig {
    host: String,
    port: u16,
}

struct LoggingConfig {
    level: String,
    file: Option<String>,
}

struct ServerConfig { // <- composed of two focused structs instead of four flat, unrelated fields
    network: NetworkConfig,
    logging: LoggingConfig,
}

let config = ServerConfig {
    network: NetworkConfig { host: "0.0.0.0".to_string(), port: 8080 },
    logging: LoggingConfig { level: "info".to_string(), file: None },
};
println!("{}:{}", config.network.host, config.network.port);
```

## Best practices & deeper information

### Scenario: Designing a public API

A payment-processing config has grown flat fields for both the merchant
account and the retry policy; splitting them into their own structs lets
each be reused and tested independently once a second payment provider
needs the same retry-policy shape.

```
struct MerchantAccount {
    id: String,
    api_key: String,
}

struct RetryPolicy {
    max_attempts: u8,
    backoff_ms: u64,
}

struct PaymentConfig { // <- two focused structs instead of four unrelated flat fields
    merchant: MerchantAccount,
    retry: RetryPolicy,
}

impl RetryPolicy {
    fn should_retry(&self, attempt: u8) -> bool { // <- a method that only makes sense on this one concern
        attempt < self.max_attempts
    }
}

let retry = RetryPolicy { max_attempts: 3, backoff_ms: 200 };
assert!(retry.should_retry(1));
```

**Why this way:** `RetryPolicy` can now be reused verbatim by a second
payment provider's config without dragging `MerchantAccount` along with
it, and `should_retry` has an obvious, single home instead of sitting on
a struct with a dozen unrelated fields — grouping by concern rather than
by "everything this feature happens to need" keeps each piece both
readable and reusable on its own.

### Scenario: Creating a new object

Composed sub-structs each get their own sensible `Default`, so the outer
struct's constructor doesn't have to hardcode every leaf value by hand.

```
#[derive(Default)]
struct NetworkConfig {
    host: String,
    port: u16,
}

#[derive(Default)]
struct LoggingConfig {
    level: String,
    file: Option<String>,
}

#[derive(Default)]
struct ServerConfig { // <- derives Default by combining each component's own Default
    network: NetworkConfig,
    logging: LoggingConfig,
}

let config = ServerConfig::default();
println!("{}", config.network.port);
```

**Why this way:** `#[derive(Default)]` on the composed struct only works
because each field's own type already implements `Default` — exactly
the kind of small, independently-defaultable component this idiom
produces, per the
[standard library's `Default` trait docs](https://doc.rust-lang.org/std/default/trait.Default.html).

## Embedded Rust Notes

**Full support.** Grouping fields into smaller structs is a pure
compile-time reorganization with no runtime cost, so it applies
identically under `#![no_std]`. Whether a given field needs the `alloc`
crate (a `String`, a `Vec`) is a property of that individual field's
type, not of composing structs together — a `#![no_std]` config could
compose the same way using fixed-size arrays or `heapless` types
instead.

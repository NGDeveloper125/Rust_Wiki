<!--
  THE CRATE DIRECTORY'S INTRODUCTION.

  This file is the prose that opens `crates/index.html`, above the directory
  itself. It has no frontmatter and is not a crate page: the leading `_` keeps
  the site generator from treating it as one.

  It is about crates as the *ecosystem* sees them — where they come from, how
  one is published, how to judge one before depending on it. What a crate is
  to the *compiler* belongs on the Crates concept page, which this links to
  rather than repeats.
-->

Almost nothing in Rust ships in the standard library. There is no HTTP client
in `std`, no JSON parser, no async runtime, no random number generator. That
looks like an omission until you notice what replaced it: a registry where any
of those is one line in a
[Cargo.toml](../concepts/modules-crates-visibility/cargo-and-cargo-toml.md)
away, and where the library you reach for is usually maintained by people who
work on that problem and nothing else.

A [crate](../concepts/modules-crates-visibility/crates.md) is what the
compiler builds in one go and what Cargo publishes as one unit. Publishing one
is deliberately undramatic: write a library, fill in a few fields of metadata,
run `cargo publish`, and it is on [crates.io](https://crates.io) under a name
nobody else can take, at a version nobody can later change. Releases are
immutable — a version that exists keeps existing, byte for byte — which is why
a build that worked last year still works today, and why
[semver](../concepts/modules-crates-visibility/dependency-management-and-semver.md)
is load-bearing rather than advisory: the version number is the whole contract
about what a new release is allowed to break.

That low barrier is why so much of Rust's vocabulary lives outside the
language. `serde` decided how Rust types are serialized; `tokio` decided what
an async runtime looks like; `rand`, `regex` and `log` each became the obvious
answer without ever being blessed. None of them are standard, and all of them
are effectively universal. When a crate becomes the common answer, the crates
built on top of it interoperate for free — a logging crate and a web framework
that both speak `log` need no adapter between them.

The cost is that choosing is now your job. A crate with millions of downloads
may still be a thin wrapper somebody abandoned in 2019, and two crates solving
the same problem can differ enormously in what they ask of you — whether they
need an async runtime, whether they work without `std`, how much of their API
is [`unsafe`](../concepts/memory-unsafe/unsafe-rust.md), how often they break.
These pages exist to make that judgement faster: what the crate is, the
situations it actually fits, and a map of its API with a call example for
every item.

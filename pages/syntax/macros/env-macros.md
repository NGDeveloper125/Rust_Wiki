---
title: "env! / option_env!"
kind: macro
embedded_support: full
groups: ["Compile-time Introspection", "Macros & Metaprogramming"]
related_concepts: []
related_syntax: ["include! / include_str! / include_bytes!"]
see_also: ["include! / include_str! / include_bytes!"]
---

## Explanation

`env!("VAR_NAME")` reads an environment variable at **compile time** —
during the build itself — and embeds its value as a `&'static str` baked
directly into the compiled binary; if the variable isn't set in the
build environment, this is a compile error, not a runtime one.
`option_env!("VAR_NAME")` is the non-panicking counterpart: instead of
failing the build, it evaluates to `Option<&'static str>`, `None` if the
variable was unset at compile time.

The critical thing both share is *when* they read: at compile time, once,
in the environment the build itself ran in — not at runtime, in the
environment the compiled program later executes in. That's the opposite
of [`std::env::var`](https://doc.rust-lang.org/std/env/fn.var.html),
which reads at runtime from whatever process environment the running
program happens to be in. Confusing the two is a common mistake:
`env!("HOME")` doesn't give the current user's home directory at runtime
— it gives whatever `HOME` happened to be set to on the machine that
*compiled* the binary, frozen into it forever.

By far the most common real use is reading the Cargo-provided build-time
variables Cargo sets for every crate: `CARGO_PKG_VERSION`,
`CARGO_PKG_NAME`, `CARGO_PKG_AUTHORS`, `CARGO_PKG_DESCRIPTION`, and
others, all populated straight from `Cargo.toml` —
`env!("CARGO_PKG_VERSION")` is the standard way to embed a crate's own
version number into itself without hand-maintaining it in two places.

## Usage examples

### Reading a crate's version and an optional build tag

```
const VERSION: &str = env!("CARGO_PKG_VERSION");          // <- fails the build if this var is somehow unset
const BUILD_TAG: Option<&str> = option_env!("BUILD_TAG"); // <- None if unset, instead of failing the build
```

### Designing a public API

A CLI tool's `--version` flag prints the crate's own version, embedded at
compile time from `Cargo.toml` rather than hand-maintained as a separate
string literal.

```
const VERSION: &str = env!("CARGO_PKG_VERSION"); // <- read from Cargo.toml's [package] version at compile time
const NAME: &str = env!("CARGO_PKG_NAME");

fn print_version() {
    println!("{NAME} {VERSION}");
}

fn handle_args(args: &[String]) {
    if args.iter().any(|arg| arg == "--version") {
        print_version();
    }
}
```

Sourcing the version string from `CARGO_PKG_VERSION`
instead of a separately hand-written constant means `Cargo.toml`'s
`version` field stays the single source of truth — the
[Cargo book](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates)
documents these variables specifically so crates don't have to duplicate
their own metadata.

### Handling and propagating errors

A build optionally embeds a license key at compile time via a build-time
environment variable; when it's absent, the binary falls back to a
"trial mode" rather than failing to compile.

```
const LICENSE_KEY: Option<&str> = option_env!("APP_LICENSE_KEY"); // <- None if the var wasn't set when this was built

fn license_status() -> &'static str {
    match LICENSE_KEY {
        Some(_) => "licensed",
        None => "trial mode", // <- graceful fallback instead of a failed build
    }
}
```

The
[std docs](https://doc.rust-lang.org/std/macro.option_env.html) draw the
line here — a genuinely optional build-time value should use
`option_env!` rather than `env!`, since `env!` turns a missing variable
into a hard compile failure, which is the wrong behavior for something
that has a sensible fallback.

## Explanation (Embedded)

Both macros are pure compile-time constructs with no dependency on `std`
or an OS, so they work identically in embedded builds. `CARGO_PKG_*`
variables in particular are just as common in firmware as in hosted
code, arguably more valuable there: without an OS-level package manager
or `--version` flag, baking `env!("CARGO_PKG_VERSION")` into the binary —
often reported back over a debug UART or RTT channel at boot — is
frequently the only way to identify which firmware image is actually
running on a device out in the field.

What's genuinely common in firmware specifically is using `env!` for
build-time paths that a `build.rs` script computed and exported, most
often `env!("OUT_DIR")` pointing at a file the build script generated —
a chip-specific memory layout, or (see
[`include! / include_str! / include_bytes!`](include-macros.md)) a whole
block of generated register-definition source that gets spliced in with
`include!` rather than checked into the repository by hand.

## Usage examples (Embedded)

### Reporting firmware version and build provenance at boot

```
const FW_VERSION: &str = env!("CARGO_PKG_VERSION");      // <- baked in at compile time from Cargo.toml
const GIT_HASH: Option<&str> = option_env!("GIT_HASH");  // <- None unless the build pipeline set this env var

fn report_boot_info() {
    match GIT_HASH {
        Some(hash) => defmt::info!("firmware {} ({})", FW_VERSION, hash),
        None => defmt::info!("firmware {} (dev build)", FW_VERSION),
    }
}
```

### Locating a build-script-generated file by path

```
// A build.rs script writes a generated register-address table under OUT_DIR;
// the crate locates it at compile time via env!, then splices it in with include!.
include!(concat!(env!("OUT_DIR"), "/generated_registers.rs")); // <- env! resolved at compile time, not at flash time
```

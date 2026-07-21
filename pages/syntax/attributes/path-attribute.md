---
title: "#[path = \"...\"]"
kind: attribute
embedded_support: full
groups: ["Modules & Visibility", "Modules, Crates & Visibility"]
related_concepts: [Modules]
related_syntax: [mod]
see_also: [mod]
---

## Explanation

`#[path = "..."]` overrides where a `mod name;` declaration looks for its
source file on disk, replacing the default `name.rs` / `name/mod.rs`
convention with an explicit path, resolved relative to the directory of
the file containing the attribute — `#[path = "impl/windows.rs"] mod
platform;` loads `platform`'s contents from `impl/windows.rs` instead of
the default `platform.rs`. The attribute can also sit on a `mod name { ... }` block
that itself contains further `mod other;` declarations, changing the base
directory those nested declarations resolve from — but in practice it's
almost always attached to a `mod name;` file-loading declaration, which is
the form nearly every real use targets.

This is a rare attribute: most crates never need it, because letting the
on-disk layout mirror the module tree — the default convention — is
simpler for any reader to navigate, and is what every other Rust codebase
already expects. It's reached for mainly in two situations: a project
whose directory layout can't or doesn't want to mirror the module tree
(most commonly, picking a different implementation file per target,
`#[path = "sys/windows.rs"] mod sys;` versus
`#[path = "sys/unix.rs"] mod sys;`, each behind its own `#[cfg(...)]`), or
loading a file generated at build time into an ordinary module path. See
[Modules](../../concepts/modules-crates-visibility/modules.md) for how
the default file-lookup convention works; this page covers only the
override.

## Usage examples

### Selecting a platform-specific file with #[cfg] and #[path]

```
#[cfg(target_os = "windows")]
#[path = "backend_windows.rs"]  // <- overrides the default `backend.rs` lookup
mod backend;

#[cfg(not(target_os = "windows"))]
#[path = "backend_unix.rs"]     // <- same module name, a different file on other platforms
mod backend;
```

### Designing a public API

A cross-platform crate exposes one `backend` module to the rest of its
code, but the actual implementation file differs per target OS —
`#[path]` points `mod backend;` at whichever file matches the platform
being compiled for, so callers see one stable module regardless of
platform.

```
// src/lib.rs
#[cfg(windows)]
#[path = "backend/windows.rs"] // <- overrides `backend.rs`/`backend/mod.rs`; only compiled on Windows
mod backend;

#[cfg(not(windows))]
#[path = "backend/unix.rs"]    // <- same module name, routed to a different file elsewhere
mod backend;

pub use backend::current_user_home; // one stable public path regardless of which file was loaded

// src/backend/windows.rs
pub fn current_user_home() -> String {
    std::env::var("USERPROFILE").unwrap_or_default()
}

// src/backend/unix.rs
pub fn current_user_home() -> String {
    std::env::var("HOME").unwrap_or_default()
}
```

Pairing `#[path]` with `#[cfg(...)]` keeps one logical
`backend` module name across platforms while routing to a genuinely
different file per target, rather than `#[cfg]`-gating code inline inside
a single shared file — the exact override behavior is documented in the
[Rust Reference's module path attribute section](https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute).

## Embedded Rust Notes

**Full support.** `#[path]` is a purely compile-time source-file
resolution attribute, so it works identically in a `#![no_std]` crate —
firmware crates commonly use the same `#[cfg]` + `#[path]` pairing shown
above to pick a different low-level implementation file per target chip.

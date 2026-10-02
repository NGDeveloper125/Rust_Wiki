<!--
  THE cli_lib TEMPLATE'S OWN PAGE.

  What this template lays out and which settings it gets right. Technical
  register, no case-making — the general argument for templates lives in
  pages/more/_templates.md.

  Same two rules as that file: no code fences, and links are relative paths to
  built pages (../../concepts/….html).
-->

`cargo new` cannot produce this layout: `--bin` and `--lib` are mutually
exclusive. The layout matters because a binary target cannot be
[integration-tested](../../concepts/testing-tooling/integration-tests.html) —
`tests/` links against a library target, so code in `main.rs` is reachable only
from unit tests in that same file. Putting the logic in a
[library crate](../../concepts/modules-crates-visibility/crates.html) and
leaving the binary as a caller is the usual way around it.

The crates sit in a
[workspace](../../concepts/modules-crates-visibility/workspaces.html) under
`crates/`. The binary declares the library as a path dependency and calls it
from `main`.

Three settings it gets right, each easy to miss when writing a workspace by
hand:

`resolver = "3"` — a virtual manifest without it falls back to resolver 1, and
cargo emits a warning on every build.

`[workspace.package]` — `version` and `edition` are declared once and taken by
each crate with `version.workspace = true`, rather than copied.

Underscores in crate names — hyphens are valid in a package name but not in a
Rust identifier, so a hyphenated package is spelled one way in `Cargo.toml` and
another in `use`.

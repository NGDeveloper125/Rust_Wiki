<!--
  THE WEBASSEMBLY SECTION'S OPENING PAGE.

  Rendered as `wasm/index.html` by tools/sitegen/src/wasm.rs. The section is
  expected to grow into an index plus pages; until it does, this file is the
  section.

  Two rules for editing this file:

  - No code fences. Every fence on this site is painted by the Rust
    highlighter, and much of what this page would quote is TOML or a shell
    command. Inline `code` is fine.
  - Links are ordinary relative paths to built pages (../concepts/….html),
    not the `.md` links used elsewhere. This file does not go through the
    link rewriter.

  Versions and dates below are checked facts. If you update them, check them
  again — crates.io is the source.
-->

## What WebAssembly is

WebAssembly is a binary instruction format for a stack-based virtual machine.
It is a compilation target, not a language: you write Rust, C, Zig or Go and
emit a `.wasm` module, which a host then loads and runs.

A module is sandboxed by construction. It has no ambient access to anything —
no files, no sockets, no clock, no environment — only the functions its host
explicitly hands it on instantiation. Whatever a module can do, the host
granted.

Its own type system is narrow. The values that cross the boundary are
integers and floats, plus opaque references in later versions. There is no
string, no struct, no vector. A module also has a *linear memory*: one flat,
growable array of bytes the host can read and write. Anything larger than a
number crosses as an offset and a length into that memory, and both sides
have to agree on what the bytes mean.

Rust suits the target unusually well, because a Rust binary carries no runtime
to port. There is no garbage collector to compile in, no interpreter, no
scheduler. A Rust wasm module is the compiled code and little else, which is
why the smallest useful module measures in hundreds of bytes.

## The two target families

Rust compiles to wasm through more than one target, and they are not variants
of each other. They assume different hosts.

`wasm32-unknown-unknown` assumes nothing. There is no platform underneath, so
the parts of the standard library that need one — threads, the filesystem,
`Instant::now` — compile but fail or panic at runtime. The host supplies
everything, and in practice the host is JavaScript. This is the browser
target.

`wasm32-wasip1` and `wasm32-wasip2` assume WASI, a standardised set of host
functions that give a module a POSIX-shaped surface: arguments, environment,
clocks, random bytes, files, sockets. Much of `std` works. A binary built for
these runs under a standalone runtime such as Wasmtime, with no JavaScript
anywhere. `wasm32-wasip2` targets the component model, the newer interface
format WASI is built on.

`wasm32-unknown-emscripten` and `wasm32v1-none` also exist, for an Emscripten
host and for bare wasm without `std` respectively.

Which family you want decides everything else — the
[crate](../concepts/modules-crates-visibility/crates.html) type, the tooling,
and whether the word "wasm" in a crate's README means your situation at all.

## How Rust's wasm tooling got here

The browser target needs more than a compiler. `wasm32-unknown-unknown`
passes numbers, so calling a Rust function that takes a string from
JavaScript means writing bytes into linear memory, passing an offset and a
length, and decoding on the other side. It is the same problem as any
other [foreign function boundary](../concepts/memory-unsafe/ffi.html),
and doing it by hand for every function and every type is what the
ecosystem set out to remove.

The Rust and WebAssembly Working Group built the answer: `wasm-bindgen`, which
generates that marshalling from an attribute on a function, and `wasm-pack`,
which drives a build and packages the result for JavaScript consumers.
Alongside them came `js-sys` and `web-sys` for the browser APIs, `gloo` for
higher-level wrappers, `twiggy` for inspecting module size, and *The Rust and
WebAssembly Book*. All of it lived in the `rustwasm` GitHub organisation.

The working group then went quiet, and stayed quiet for about five years. In
July 2025 the organisation was
[formally sunset](https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/).
The stated reason was that leaving it open had become misleading: it was not
clear to anyone, users or maintainers, which repositories were still alive.
Archiving it made `rustwasm` what it had in practice become — a historical
record.

The pieces did not all go the same way.

`wasm-bindgen` was transferred to an organisation of its own, with new
maintainers, and is actively developed: version 0.2.129 was published in
September 2026. Its guide, not the archived book, is the current reference.

`wasm-pack` was claimed and moved to that same organisation. Its release
history shows the gap plainly — 0.13.1 in October 2024, nothing through the
sunset, then 0.14.0 in January 2026 and 0.15.0 in May 2026. It is maintained
again, and `wasm-bindgen`'s own documentation names it as the tool for
packaging a module.

Everything else was archived in place unless a maintainer claimed it. `gloo`,
`twiggy` and the book were not. The book is still the first result most
searches return, and it now carries a notice saying it is unmaintained.

That history is worth knowing for a practical reason rather than a
sentimental one: a great deal of writing about Rust and WebAssembly predates
the sunset, and points at repositories and documentation that no longer move.
Some widely used crates are in the archive too — `console_error_panic_hook`,
which almost every browser-targeting project uses to get a readable panic
message, last saw a release in 2021 and is still downloaded millions of times
a month.

## Where things stand

For a wasm module called from JavaScript, the maintained path is
`wasm-bindgen` for the boundary and `wasm-pack` to build and package it. That
is the combination its own documentation points to, and both are actively
released.

For a whole user interface written in Rust, the frameworks are the entry
point rather than the tooling: Leptos, Dioxus and Yew, each with its own
build command. `trunk`, once the default bundler for this kind of project, is
now comparatively little used.

For code that is not aimed at a browser at all, the WASI targets and a
standalone runtime replace the entire JavaScript story, and none of
`wasm-bindgen`, `wasm-pack` or the frameworks above are involved.
